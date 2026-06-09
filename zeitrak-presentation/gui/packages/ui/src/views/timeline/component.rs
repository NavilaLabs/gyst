use std::collections::HashMap;

use chrono::{DateTime, Duration, TimeZone, Utc};
use dioxus::prelude::*;
use dioxus_i18n::tid;

use crate::{
    components::molecules::MemberFilter, formatting, layouts::DefaultLayout, ActivitiesCache,
    RunningElapsed, RunningTimer, UserSettings,
};

use super::{
    rail::{GapSegment, RailSegment, TimelineRailColumn},
    station::{StationEntry, TimelineStation},
};

// ── Aggregation ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Aggregation {
    Individual,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Aggregation {
    fn px_per_min(self) -> f64 {
        match self {
            Self::Individual | Self::Hour => 2.0,
            Self::Day => 1.0,
            Self::Week => 0.25,
            Self::Month => 0.07,
            Self::Year => 0.015,
        }
    }

    fn schwelle_interval_min(self) -> i64 {
        match self {
            Self::Individual => 60, // 1 h
            Self::Hour => 60,       // 1 h
            Self::Day => 1440,      // 1 day
            Self::Week => 10_080,   // 1 week
            Self::Month => 43_200,  // ~30 days
            Self::Year => 525_600,  // ~365 days
        }
    }

    fn group_key(self, rfc3339: &str) -> String {
        let Ok(dt) = DateTime::parse_from_rfc3339(rfc3339) else {
            return rfc3339.to_string();
        };
        let dt = dt.with_timezone(&Utc);
        match self {
            Self::Individual => rfc3339.to_string(),
            Self::Hour => dt.format("%Y-%m-%dT%H").to_string(),
            Self::Day => dt.format("%Y-%m-%d").to_string(),
            Self::Week => {
                use chrono::Datelike;
                format!("{}-W{:02}", dt.year(), dt.iso_week().week())
            }
            Self::Month => dt.format("%Y-%m").to_string(),
            Self::Year => dt.format("%Y").to_string(),
        }
    }
}

// ── Station ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Station {
    start_time: String,
    end_time: Option<String>,
    total_seconds: i64,
    entries: Vec<api::timesheet::TimesheetDto>,
    is_running: bool,
}

impl Station {
    fn dot_y(&self, t_max: DateTime<Utc>, px_per_min: f64) -> f64 {
        let Ok(dt) = DateTime::parse_from_rfc3339(&self.start_time) else {
            return 0.0;
        };
        let mins = (t_max - dt.with_timezone(&Utc)).num_minutes();
        (mins as f64) * px_per_min
    }

    fn segment_top_y(&self, t_max: DateTime<Utc>, px_per_min: f64) -> f64 {
        let end = self
            .end_time
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(t_max);
        let mins = (t_max - end).num_minutes();
        (mins as f64) * px_per_min
    }

    fn segment_height(&self, px_per_min: f64) -> f64 {
        (self.total_seconds as f64 / 60.0) * px_per_min
    }

    fn primary_color<'a>(&self, color_map: &'a HashMap<String, String>) -> &'a str {
        self.entries
            .iter()
            .find_map(|e| {
                e.activity_id
                    .as_deref()
                    .and_then(|id| color_map.get(id))
                    .map(String::as_str)
            })
            .unwrap_or("#6c6c76")
    }

    fn is_single_activity(&self) -> bool {
        let unique: std::collections::HashSet<_> = self
            .entries
            .iter()
            .filter_map(|e| e.activity_id.as_deref())
            .collect();
        unique.len() <= 1
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn group_into_stations(entries: &[api::timesheet::TimesheetDto], agg: Aggregation) -> Vec<Station> {
    let mut order: Vec<String> = Vec::new();
    let mut map: HashMap<String, Vec<api::timesheet::TimesheetDto>> = HashMap::new();

    for e in entries {
        let key = agg.group_key(&e.start_time);
        if !map.contains_key(&key) {
            order.push(key.clone());
        }
        map.entry(key).or_default().push(e.clone());
    }

    order
        .into_iter()
        .map(|key| {
            let group = map.remove(&key).unwrap_or_default();
            let start_time = group
                .iter()
                .map(|e| e.start_time.clone())
                .min()
                .unwrap_or_default();
            let is_running = group.iter().any(|e| e.end_time.is_none());
            let end_time = if is_running {
                None
            } else {
                group.iter().map(|e| e.end_time.clone()).max().flatten()
            };
            let total_seconds = group.iter().filter_map(|e| e.duration).map(i64::from).sum();
            Station {
                start_time,
                end_time,
                total_seconds,
                entries: group,
                is_running,
            }
        })
        .collect()
}

fn fmt_duration(secs: i64) -> String {
    if secs <= 0 {
        return "0m".to_string();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    match (h, m) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

fn build_maps(
    activities: &[api::activity::ActivityDto],
) -> (HashMap<String, String>, HashMap<String, String>) {
    let colors = activities
        .iter()
        .map(|a| (a.id.clone(), a.color.clone()))
        .collect();
    let names = activities
        .iter()
        .map(|a| (a.id.clone(), a.name.clone()))
        .collect();
    (colors, names)
}

// ── Segment parts for multi-activity coloring ─────────────────────────────────

fn segment_parts(
    station: &Station,
    color_map: &HashMap<String, String>,
    total_height: f64,
) -> Vec<(String, f64)> {
    if station.is_single_activity() {
        return vec![(station.primary_color(color_map).to_string(), total_height)];
    }
    let total_dur: f64 = station
        .entries
        .iter()
        .filter_map(|e| e.duration)
        .map(|d| d as f64)
        .sum();
    if total_dur <= 0.0 {
        return vec![("#6c6c76".to_string(), total_height)];
    }
    station
        .entries
        .iter()
        .filter_map(|e| {
            let dur = e.duration? as f64;
            let color = e
                .activity_id
                .as_deref()
                .and_then(|id| color_map.get(id))
                .cloned()
                .unwrap_or_else(|| "#6c6c76".to_string());
            Some((color, (dur / total_dur) * total_height))
        })
        .collect()
}

// ── Page ──────────────────────────────────────────────────────────────────────

#[component]
pub fn Timeline() -> Element {
    let mut from: Signal<String> = use_signal(String::new);
    let mut to: Signal<String> = use_signal(String::new);
    let mut aggregation: Signal<Aggregation> = use_signal(|| Aggregation::Individual);
    let mut member_id: Signal<Option<String>> = use_signal(|| None);
    let mut can_filter_members = use_signal(|| false);
    let mut members: Signal<Vec<api::member::MemberDto>> = use_signal(Vec::new);
    let mut entries: Signal<Vec<api::timesheet::TimesheetDto>> = use_signal(Vec::new);
    let mut current_page: Signal<u32> = use_signal(|| 0u32);
    let mut has_more: Signal<bool> = use_signal(|| true);
    let mut loading: Signal<bool> = use_signal(|| false);

    let running_timer: RunningTimer = use_context();
    let elapsed: RunningElapsed = use_context();
    let activities_cache: ActivitiesCache = use_context();
    let user_settings: UserSettings = use_context();
    let mut toasts: crate::components::atoms::Toasts = use_context();

    let stats = use_resource(move || {
        let f = from();
        let t = to();
        let mid = member_id();
        async move {
            api::timesheet::get_timeline_stats(
                if f.is_empty() { None } else { Some(f) },
                if t.is_empty() { None } else { Some(t) },
                mid,
            )
            .await
        }
    });

    let _ = use_resource(move || {
        let f = from();
        let t = to();
        let mid = member_id();
        async move {
            entries.set(vec![]);
            current_page.set(0);
            has_more.set(true);
            loading.set(true);
            match api::timesheet::get_timeline_entries(
                0,
                if f.is_empty() { None } else { Some(f) },
                if t.is_empty() { None } else { Some(t) },
                mid,
            )
            .await
            {
                Ok(page) => {
                    if page.can_filter_members && members.peek().is_empty() {
                        if let Ok(list) = api::member::list_members().await {
                            members.set(list);
                        }
                    }
                    can_filter_members.set(page.can_filter_members);
                    let loaded = page.items.len() as u64;
                    has_more.set(loaded < page.total);
                    entries.set(page.items);
                    current_page.set(1);
                }
                Err(e) => toasts
                    .write()
                    .push(crate::components::atoms::ToastMessage::error(e.to_string())),
            }
            loading.set(false);
        }
    });

    let on_member_change = move |new_member: Option<String>| {
        member_id.set(new_member);
    };

    let load_more = move |_| {
        spawn(async move {
            if *loading.peek() || !*has_more.peek() {
                return;
            }
            loading.set(true);
            let page = *current_page.peek();
            let f = from.peek().clone();
            let t = to.peek().clone();
            let mid = member_id.peek().clone();
            match api::timesheet::get_timeline_entries(
                page,
                if f.is_empty() { None } else { Some(f) },
                if t.is_empty() { None } else { Some(t) },
                mid,
            )
            .await
            {
                Ok(result) => {
                    let new_total = entries.read().len() as u64 + result.items.len() as u64;
                    has_more.set(new_total < result.total);
                    entries.write().extend(result.items);
                    current_page.set(page + 1);
                }
                Err(e) => toasts
                    .write()
                    .push(crate::components::atoms::ToastMessage::error(e.to_string())),
            }
            loading.set(false);
        });
    };

    let (color_map, name_map) = build_maps(&activities_cache.read());
    let tz = user_settings.read().timezone.clone();
    let date_fmt = user_settings.read().date_format.clone();
    let elapsed_secs = *elapsed.read();

    let agg = *aggregation.read();
    let px_per_min = agg.px_per_min();
    let t_max = Utc::now();

    let all_entries = entries.read().clone();
    let mut stations = group_into_stations(&all_entries, *aggregation.read());

    if let Some(running) = running_timer.read().clone() {
        if !all_entries.iter().any(|e| e.id == running.id) {
            // Use actual wall-clock diff so segment_height stays in sync with dot_y.
            // elapsed is kept only for the live display in the card footer.
            let secs = DateTime::parse_from_rfc3339(&running.start_time)
                .ok()
                .map(|dt| (t_max - dt.with_timezone(&Utc)).num_seconds().max(0))
                .unwrap_or(elapsed_secs as i64);
            stations.insert(
                0,
                Station {
                    start_time: running.start_time.clone(),
                    end_time: None,
                    total_seconds: secs,
                    entries: vec![running],
                    is_running: true,
                },
            );
        }
    }

    let t_min = stations
        .iter()
        .filter_map(|s| DateTime::parse_from_rfc3339(&s.start_time).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .min()
        .unwrap_or_else(|| t_max - Duration::hours(1));

    let total_min = (t_max - t_min).num_minutes().max(60);
    let track_height = (total_min as f64 * px_per_min + 120.0) as i64;

    let schwelle_interval = agg.schwelle_interval_min();
    // Start Schwellen from the top of the first rail segment, not from the dot.
    // Using dot_y would hide all Schwellen inside the topmost colored segment.
    let first_segment_top_y = stations
        .first()
        .map(|s| s.segment_top_y(t_max, px_per_min))
        .unwrap_or(0.0);
    let schwellen: Vec<f64> = {
        let mut result = Vec::new();
        let epoch_mins = t_max.timestamp() / 60;
        let rounded_mins = (epoch_mins / schwelle_interval) * schwelle_interval;
        let mut t = Utc
            .timestamp_opt(rounded_mins * 60, 0)
            .single()
            .unwrap_or(t_max);
        let bottom_limit = t_min - Duration::minutes(schwelle_interval);
        while t >= bottom_limit {
            let y = (t_max - t).num_minutes() as f64 * px_per_min;
            if y >= first_segment_top_y {
                result.push(y);
            }
            t -= Duration::minutes(schwelle_interval);
        }
        result
    };

    // Precompute rail segments and gaps
    let mut rail_segments: Vec<RailSegment> = Vec::new();
    let mut gap_segments: Vec<GapSegment> = Vec::new();

    for (idx, station) in stations.iter().enumerate() {
        let seg_top = station.segment_top_y(t_max, px_per_min);
        let seg_h = station.segment_height(px_per_min).max(2.0);
        let parts = segment_parts(station, &color_map, seg_h);
        let mut part_y = seg_top;
        for (color, h) in parts {
            let h = h.max(1.0);
            rail_segments.push(RailSegment {
                top: part_y,
                height: h,
                color,
            });
            part_y += h;
        }

        let dot_y = station.dot_y(t_max, px_per_min);

        // Intra-station gap: aggregated entries may have internal free time between
        // the last end_time and the earliest start_time (e.g. two separate activities
        // in the same day).  Without this the dot appears disconnected from the rail.
        let intra_gap_h = dot_y - part_y;
        if intra_gap_h > 1.0 {
            gap_segments.push(GapSegment {
                top: part_y,
                height: intra_gap_h,
            });
        }

        // Gap from this station's dot down to the next station's segment top
        if let Some(next) = stations.get(idx + 1) {
            let next_seg_top = next.segment_top_y(t_max, px_per_min);
            let gap_h = next_seg_top - dot_y;
            if gap_h > 1.0 {
                gap_segments.push(GapSegment {
                    top: dot_y,
                    height: gap_h,
                });
            }
        }
    }
    // Tail below last station
    if let Some(last) = stations.last() {
        let tail_top = last.dot_y(t_max, px_per_min);
        let tail_h = track_height as f64 - tail_top;
        if tail_h > 0.0 {
            gap_segments.push(GapSegment {
                top: tail_top,
                height: tail_h,
            });
        }
    }

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        DefaultLayout {
            div { class: "tl-page",

                // ── Metrics bar ───────────────────────────────────────────────
                div { class: "tl-metrics",
                    div { class: "tl-metrics-top",
                        div { class: "tl-filter-row",
                            div { class: "tl-filter-group",
                                label { class: "tl-filter-label", {tid!("timeline-filter-from")} }
                                input {
                                    r#type: "date",
                                    class: "tl-date-input",
                                    value: "{from}",
                                    oninput: move |e| from.set(e.value()),
                                }
                            }
                            div { class: "tl-filter-group",
                                label { class: "tl-filter-label", {tid!("timeline-filter-to")} }
                                input {
                                    r#type: "date",
                                    class: "tl-date-input",
                                    value: "{to}",
                                    oninput: move |e| to.set(e.value()),
                                }
                            }
                            if *can_filter_members.read() {
                                MemberFilter {
                                    members,
                                    selected: member_id,
                                    on_change: on_member_change,
                                }
                            }
                            if !from.read().is_empty() || !to.read().is_empty() {
                                button {
                                    class: "tl-filter-clear",
                                    onclick: move |_| {
                                        from.set(String::new());
                                        to.set(String::new());
                                    },
                                    {tid!("timeline-filter-clear")}
                                }
                            }
                        }
                        div { class: "tl-agg-tabs",
                            for (label, level) in [
                                (tid!("timeline-aggregation-individual"), Aggregation::Individual),
                                (tid!("timeline-aggregation-hour"),       Aggregation::Hour),
                                (tid!("timeline-aggregation-day"),        Aggregation::Day),
                                (tid!("timeline-aggregation-week"),       Aggregation::Week),
                                (tid!("timeline-aggregation-month"),      Aggregation::Month),
                                (tid!("timeline-aggregation-year"),       Aggregation::Year),
                            ] {
                                button {
                                    class: if *aggregation.read() == level {
                                        "tab-pill tab-pill--active"
                                    } else {
                                        "tab-pill"
                                    },
                                    onclick: move |_| aggregation.set(level),
                                    "{label}"
                                }
                            }
                        }
                    }

                    if let Some(Ok(s)) = &*stats.read_unchecked() {
                        div { class: "tl-metrics-stats",
                            div { class: "tl-stat-block",
                                span { class: "tl-stat-label", {tid!("timeline-metrics-total")} }
                                span { class: "tl-stat-value font-mono", {fmt_duration(s.total_seconds)} }
                            }
                            div { class: "tl-activity-breakdown",
                                for act in &s.by_activity {
                                    div { class: "tl-act-row",
                                        span {
                                            class: "zk-activity-dot",
                                            style: "background:{act.color}",
                                        }
                                        span { class: "tl-act-name", "{act.activity_name}" }
                                        span { class: "tl-act-dur font-mono", {fmt_duration(act.total_seconds)} }
                                        div { class: "tl-act-bar-wrap",
                                            div {
                                                class: "tl-act-bar",
                                                style: "width:{act.percentage:.0}%;background:{act.color}",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Track scroll area ─────────────────────────────────────────
                div { class: "tl-scroll",
                    if stations.is_empty() && !*loading.read() {
                        div { class: "tl-empty", {tid!("timeline-no-entries")} }
                    } else {
                        div {
                            class: "tl-inner",
                            style: "min-height:{track_height}px",

                            // ── Rail column ───────────────────────────────────
                            TimelineRailColumn { schwellen, segments: rail_segments, gaps: gap_segments }

                            // ── Stations ──────────────────────────────────────
                            for (idx, station) in stations.iter().enumerate() {
                                {
                                    let dot_y = station.dot_y(t_max, px_per_min);
                                    let side = if idx % 2 == 0 { "left" } else { "right" };
                                    let is_running = station.is_running;
                                    let is_single = station.is_single_activity();
                                    let dot_color = if is_single {
                                        station.primary_color(&color_map).to_string()
                                    } else {
                                        "#6c6c76".to_string()
                                    };
                                    let total_secs = if is_running {
                                        elapsed_secs as i64
                                    } else {
                                        station.total_seconds
                                    };
                                    let title = if station.entries.len() == 1 {
                                        station.entries[0]
                                            .activity_id
                                            .as_deref()
                                            .and_then(|id| name_map.get(id))
                                            .cloned()
                                            .unwrap_or_else(|| {
                                                formatting::format_date(
                                                    &station.entries[0].start_time,
                                                    &tz,
                                                    &date_fmt,
                                                )
                                            })
                                    } else {
                                        formatting::format_date(
                                            &station.entries[0].start_time,
                                            &tz,
                                            &date_fmt,
                                        )
                                    };
                                    let station_entries: Vec<StationEntry> = station
                                        .entries
                                        .iter()
                                        .map(|e| {
                                            let activity_name: String = e
                                                .activity_id
                                                .as_deref()
                                                .and_then(|id| name_map.get(id))
                                                .cloned()
                                                .unwrap_or("--".to_string());
                                            let activity_color = e
                                                .activity_id
                                                .as_deref()
                                                .and_then(|id| color_map.get(id))
                                                .cloned();
                                            let start_fmt = formatting::format_datetime(
                                                &e.start_time,
                                                &tz,
                                                &date_fmt,
                                            );
                                            let end_fmt = e.end_time.as_deref().map(|t| {
                                                formatting::format_datetime(t, &tz, &date_fmt)
                                            });
                                            let time_range = match end_fmt {
                                                Some(end) => format!("{start_fmt} \u{2013} {end}"),
                                                None => start_fmt,
                                            };
                                            let entry_is_running =
                                                e.end_time.is_none() && station.is_running;
                                            let duration = e
                                                .duration
                                                .map(|d| fmt_duration(i64::from(d)))
                                                .or_else(|| {
                                                    entry_is_running
                                                        .then(|| fmt_duration(elapsed_secs as i64))
                                                });
                                            StationEntry {
                                                activity_name: Some(activity_name),
                                                activity_color,
                                                time_range,
                                                duration,
                                                description: e.description.clone(),
                                                is_running: entry_is_running,
                                            }
                                        })
                                        .collect();
                                    let key = station.start_time.clone();
                                    rsx! {
                                        TimelineStation {
                                            key: "{key}",
                                            dot_y,
                                            side: side.to_string(),
                                            is_running,
                                            dot_color,
                                            title,
                                            entries: station_entries,
                                            total_duration: fmt_duration(total_secs),
                                        }
                                    }
                                }
                            }

                            // Load-more
                            if *has_more.read() || *loading.read() {
                                div { class: "tl-load-more",
                                    if *loading.read() {
                                        div { class: "tl-loading-dots",
                                            span {} span {} span {}
                                        }
                                    } else {
                                        button {
                                            class: "tab-pill",
                                            onclick: load_more,
                                            {tid!("timeline-load-more")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
