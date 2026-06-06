use crate::components::atoms::card::{Card, CardContent};
use crate::components::atoms::{Button, ButtonVariant, Select, SelectOption, ToastExt, Toasts};
use crate::formatting;
use crate::layouts::DefaultLayout;
use crate::{ActivitiesCache, PluginHostCtx, TimesheetsCache};
use dioxus_extism_frontend::PluginSlot;
use chrono::{Datelike, Duration, Utc};
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiLightningBolt, HiPlay, HiStop};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

// ── Chart period ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum ChartPeriod {
    Week,
    Month,
    Year,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.date_naive())
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|dt| dt.date())
        })
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                .ok()
                .map(|dt| dt.date())
        })
}

fn fmt_hours(h: f32) -> String {
    if h < 0.1 {
        "0h".to_string()
    } else if h < 1.0 {
        format!("{:.0}m", h * 60.0)
    } else {
        format!("{:.1}h", h)
    }
}

fn fmt_hours_axis(v: f32) -> String {
    format!("{:.0}h", v)
}

fn nice_max(v: f32) -> f32 {
    if v <= 1.0 {
        1.0
    } else if v <= 2.0 {
        2.0
    } else if v <= 4.0 {
        4.0
    } else if v <= 6.0 {
        6.0
    } else if v <= 8.0 {
        8.0
    } else if v <= 12.0 {
        12.0
    } else if v <= 16.0 {
        16.0
    } else if v <= 24.0 {
        24.0
    } else {
        (v / 8.0).ceil() * 8.0
    }
}

// ── Hours bar chart with hover tooltip ───────────────────────────────────────

#[component]
fn HoursBarChart(bars: Vec<Vec<BarSegment>>, labels: Vec<String>) -> Element {
    let mut hovered: Signal<Option<usize>> = use_signal(|| None);

    let vw = 560i32;
    let vh = 240i32;
    let chart_left = 50.0f32;
    let chart_right = 550.0f32;
    let chart_top = 20.0f32;
    let chart_bottom = 204.0f32;
    let chart_w = chart_right - chart_left;
    let chart_h = chart_bottom - chart_top;

    let n = bars.len().max(1);
    let slot_w = chart_w / n as f32;
    let bar_w = (slot_w * 0.55).clamp(4.0, 60.0);

    let max_val = bars
        .iter()
        .map(|segs| segs.iter().map(|s| s.hours).sum::<f32>())
        .fold(0.0f32, f32::max);
    let y_max = nice_max(max_val);

    let ticks: Vec<f32> = (0..5).map(|i| i as f32 * y_max / 4.0).collect();

    let val_to_y = |v: f32| chart_bottom - (v / y_max) * chart_h;
    let cx_of = |i: usize| chart_left + slot_w * (i as f32 + 0.5);

    let hov = *hovered.read();

    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "100%",
            height: "100%",
            class: "dx-chart-bar",
            preserve_aspect_ratio: "xMidYMid meet",
            view_box: "0 0 {vw} {vh}",
            onmouseleave: move |_| hovered.set(None),

            // ── Y-axis grid lines + labels ─────────────────────────────────
            g { class: "dx-grid",
                for tick_v in ticks.iter() {
                    {
                        let y = val_to_y(*tick_v);
                        let lbl = fmt_hours_axis(*tick_v);
                        rsx! {
                            line {
                                x1: "{chart_left}",
                                y1: "{y:.1}",
                                x2: "{chart_right}",
                                y2: "{y:.1}",
                                class: "dx-grid-line",
                            }
                            text {
                                x: "{chart_left - 5.0}",
                                y: "{y:.1}",
                                text_anchor: "end",
                                alignment_baseline: "middle",
                                class: "dx-grid-label",
                                "{lbl}"
                            }
                        }
                    }
                }
            }

            // ── Stacked bar visuals (pointer-events: none so hit areas work) ──
            for (i, segs) in bars.iter().enumerate() {
                {
                    let total_v: f32 = segs.iter().map(|s| s.hours).sum();
                    let top_y = val_to_y(total_v);
                    let bar_h = chart_bottom - top_y;
                    let bx = cx_of(i) - bar_w / 2.0;
                    let is_hov = hov == Some(i);
                    let opacity = if is_hov { "1" } else { "0.85" };
                    let single = segs.len() == 1;

                    // Precompute (y, height, color, rx) for each segment stacked bottom→top
                    let seg_data: Vec<(f32, f32, String, &str)> = segs
                        .iter()
                        .enumerate()
                        .scan(0.0f32, |acc, (k, seg)| {
                            let h = (seg.hours / total_v) * bar_h;
                            let y = chart_bottom - *acc - h;
                            *acc += h;
                            let rx = if single || k == segs.len() - 1 { "3" } else { "0" };
                            Some((y, h, seg.color.clone(), rx))
                        })
                        .collect();

                    rsx! {
                        if bar_h > 0.5 {
                            g {
                                key: "bar-{i}",
                                opacity: "{opacity}",
                                pointer_events: "none",
                                for (sy, sh, sc, srx) in seg_data {
                                    rect {
                                        x: "{bx:.1}",
                                        y: "{sy:.1}",
                                        width: "{bar_w:.1}",
                                        height: "{sh:.1}",
                                        rx: "{srx}",
                                        fill: "{sc}",
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Hit areas (rendered on top) + X-axis labels ────────────────
            for (i, label) in labels.iter().enumerate() {
                {
                    let slot_x = chart_left + slot_w * i as f32;
                    let cx = cx_of(i);
                    rsx! {
                        rect {
                            key: "hit-{i}",
                            x: "{slot_x:.1}",
                            y: "{chart_top}",
                            width: "{slot_w:.1}",
                            height: "{chart_h}",
                            fill: "transparent",
                            cursor: "crosshair",
                            onmouseenter: move |_| hovered.set(Some(i)),
                        }
                        text {
                            x: "{cx:.1}",
                            y: "{chart_bottom + 15.0}",
                            text_anchor: "middle",
                            alignment_baseline: "middle",
                            class: "dx-bar-label",
                            "{label}"
                        }
                    }
                }
            }

            // ── Tooltip ────────────────────────────────────────────────────
            if let Some(idx) = hov {
                if idx < bars.len() {
                    {
                        let segs = &bars[idx];
                        let total_v: f32 = segs.iter().map(|s| s.hours).sum();
                        let cx = cx_of(idx);
                        let top_y = val_to_y(total_v);
                        let label = &labels[idx];

                        let n_rows = segs.len().max(1);
                        let tw = 150.0f32;
                        let th = 20.0 + 16.0 * n_rows as f32 + 8.0;
                        let tx = (cx - tw / 2.0).max(chart_left).min(chart_right - tw);
                        let ty = (top_y - th - 8.0).max(chart_top);

                        rsx! {
                            g { pointer_events: "none",
                                rect {
                                    x: "{tx:.1}",
                                    y: "{ty:.1}",
                                    width: "{tw}",
                                    height: "{th:.1}",
                                    rx: "5",
                                    class: "dx-tooltip-bg",
                                }
                                text {
                                    x: "{tx + tw / 2.0:.1}",
                                    y: "{ty + 14.0:.1}",
                                    text_anchor: "middle",
                                    alignment_baseline: "middle",
                                    class: "dx-tooltip-label",
                                    "{label}"
                                }
                                if segs.is_empty() {
                                    text {
                                        x: "{tx + tw / 2.0:.1}",
                                        y: "{ty + 30.0:.1}",
                                        text_anchor: "middle",
                                        alignment_baseline: "middle",
                                        class: "dx-tooltip-value",
                                        "0h"
                                    }
                                }
                                for (k, seg) in segs.iter().enumerate() {
                                    {
                                        let row_cy = ty + 23.5 + 16.0 * k as f32;
                                        let dot_x = tx + 10.0;
                                        let color = seg.color.clone();
                                        let pct = if total_v > 0.0 {
                                            seg.hours / total_v * 100.0
                                        } else {
                                            0.0
                                        };
                                        let name: String = if seg.name.chars().count() > 13 {
                                            let t: String = seg.name.chars().take(12).collect();
                                            format!("{t}…")
                                        } else {
                                            seg.name.clone()
                                        };
                                        let hours_str = fmt_hours(seg.hours);
                                        let pct_str = format!("{pct:.0}%");
                                        rsx! {
                                            rect {
                                                x: "{dot_x:.1}",
                                                y: "{row_cy - 3.5:.1}",
                                                width: "7",
                                                height: "7",
                                                rx: "2",
                                                fill: "{color}",
                                            }
                                            text {
                                                x: "{dot_x + 10.0:.1}",
                                                y: "{row_cy:.1}",
                                                text_anchor: "start",
                                                alignment_baseline: "middle",
                                                font_size: "9",
                                                class: "dx-tooltip-label",
                                                "{name}"
                                            }
                                            text {
                                                x: "{tx + tw - 38.0:.1}",
                                                y: "{row_cy:.1}",
                                                text_anchor: "end",
                                                alignment_baseline: "middle",
                                                font_size: "9",
                                                class: "dx-tooltip-value",
                                                "{hours_str}"
                                            }
                                            text {
                                                x: "{tx + tw - 4.0:.1}",
                                                y: "{row_cy:.1}",
                                                text_anchor: "end",
                                                alignment_baseline: "middle",
                                                font_size: "9",
                                                class: "dx-tooltip-label",
                                                "{pct_str}"
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
}

// ── Stats ─────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct ActivityMixItem {
    name: String,
    color: String,
    hours: f32,
}

#[derive(Clone, PartialEq)]
struct BarSegment {
    name: String,
    color: String,
    hours: f32,
}

struct DashStats {
    today_hours: f32,
    week_hours: f32,
    streak: u32,
    activity_mix: Vec<ActivityMixItem>,
}

fn compute_stats(
    timesheets: &[api::timesheet::TimesheetDto],
    activities: &[api::activity::ActivityDto],
) -> DashStats {
    let today = Utc::now().date_naive();
    let days_from_monday = today.weekday().num_days_from_monday() as i64;
    let week_start = today - Duration::days(days_from_monday);

    let today_hours: f32 = timesheets
        .iter()
        .filter(|ts| ts.duration.is_some() && parse_date(&ts.start_time) == Some(today))
        .map(|ts| ts.duration.unwrap_or(0) as f32 / 3600.0)
        .sum();

    let week_hours: f32 = timesheets
        .iter()
        .filter(|ts| {
            ts.duration.is_some()
                && parse_date(&ts.start_time)
                    .map(|d| d >= week_start)
                    .unwrap_or(false)
        })
        .map(|ts| ts.duration.unwrap_or(0) as f32 / 3600.0)
        .sum();

    // Streak: consecutive days with at least one completed timesheet, counting back from today
    let mut streak = 0u32;
    let mut check = today;
    loop {
        let has_entry = timesheets
            .iter()
            .any(|ts| ts.duration.is_some() && parse_date(&ts.start_time) == Some(check));
        if has_entry {
            streak += 1;
            check -= Duration::days(1);
        } else {
            break;
        }
    }

    // Activity mix
    let mut mix_map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for ts in timesheets {
        if let (Some(aid), Some(dur)) = (&ts.activity_id, ts.duration) {
            if dur > 0 {
                *mix_map.entry(aid.clone()).or_insert(0.0) += dur as f32 / 3600.0;
            }
        }
    }
    let mut activity_mix: Vec<ActivityMixItem> = mix_map
        .into_iter()
        .map(|(aid, hours)| {
            let activity = activities.iter().find(|a| a.id == aid);
            let name = activity
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "—".to_string());
            let color = activity
                .map(|a| a.color.clone())
                .unwrap_or_else(|| "#6c6c76".to_string());
            ActivityMixItem { name, color, hours }
        })
        .collect();
    activity_mix.sort_by(|a, b| {
        b.hours
            .partial_cmp(&a.hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    DashStats {
        today_hours,
        week_hours,
        streak,
        activity_mix,
    }
}

fn bucket_to_segments(
    timesheets: &[&api::timesheet::TimesheetDto],
    activities: &[api::activity::ActivityDto],
) -> Vec<BarSegment> {
    let mut map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for ts in timesheets {
        if let Some(dur) = ts.duration {
            if dur > 0 {
                let key = ts.activity_id.clone().unwrap_or_default();
                *map.entry(key).or_insert(0.0) += dur as f32 / 3600.0;
            }
        }
    }
    let mut segs: Vec<BarSegment> = map
        .into_iter()
        .map(|(aid, hours)| {
            let activity = activities.iter().find(|a| a.id == aid);
            let name = activity
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "—".to_string());
            let color = activity
                .map(|a| a.color.clone())
                .unwrap_or_else(|| "#6c6c76".to_string());
            BarSegment { name, color, hours }
        })
        .collect();
    segs.sort_by(|a, b| {
        b.hours
            .partial_cmp(&a.hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    segs
}

fn compute_chart_data(
    timesheets: &[api::timesheet::TimesheetDto],
    activities: &[api::activity::ActivityDto],
    period: &ChartPeriod,
) -> (Vec<Vec<BarSegment>>, Vec<String>) {
    let today = Utc::now().date_naive();

    match period {
        ChartPeriod::Week => {
            let bars: Vec<Vec<BarSegment>> = (0..7)
                .map(|i| {
                    let day = today - Duration::days(6 - i as i64);
                    let bucket: Vec<&api::timesheet::TimesheetDto> = timesheets
                        .iter()
                        .filter(|ts| {
                            ts.duration.is_some() && parse_date(&ts.start_time) == Some(day)
                        })
                        .collect();
                    bucket_to_segments(&bucket, activities)
                })
                .collect();
            let labels: Vec<String> = (0..7)
                .map(|i| {
                    let day = today - Duration::days(6 - i as i64);
                    day.format("%a").to_string()
                })
                .collect();
            (bars, labels)
        }
        ChartPeriod::Month => {
            // 4 weekly buckets covering the last 28 days
            let bars: Vec<Vec<BarSegment>> = (0..4)
                .map(|w| {
                    let week_end = today - Duration::days((3 - w as i64) * 7);
                    let week_start = week_end - Duration::days(6);
                    let bucket: Vec<&api::timesheet::TimesheetDto> = timesheets
                        .iter()
                        .filter(|ts| {
                            ts.duration.is_some()
                                && parse_date(&ts.start_time)
                                    .map(|d| d >= week_start && d <= week_end)
                                    .unwrap_or(false)
                        })
                        .collect();
                    bucket_to_segments(&bucket, activities)
                })
                .collect();
            let labels: Vec<String> = (0..4)
                .map(|w| {
                    let week_end = today - Duration::days((3 - w as i64) * 7);
                    let week_start = week_end - Duration::days(6);
                    format!("{}", week_start.format("%-d.%-m"))
                })
                .collect();
            (bars, labels)
        }
        ChartPeriod::Year => {
            // 12 monthly bars
            let bars: Vec<Vec<BarSegment>> = (0..12)
                .map(|m| {
                    let month_offset = 11 - m;
                    let target_year =
                        today.year() - (month_offset + 12 - today.month() as i32).max(0) / 12;
                    let target_month =
                        ((today.month() as i32 - month_offset - 1).rem_euclid(12) + 1) as u32;
                    let bucket: Vec<&api::timesheet::TimesheetDto> = timesheets
                        .iter()
                        .filter(|ts| {
                            ts.duration.is_some()
                                && parse_date(&ts.start_time)
                                    .map(|d| d.year() == target_year && d.month() == target_month)
                                    .unwrap_or(false)
                        })
                        .collect();
                    bucket_to_segments(&bucket, activities)
                })
                .collect();
            let labels: Vec<String> = (0..12)
                .map(|m| {
                    let month_offset = 11 - m;
                    let target_month =
                        ((today.month() as i32 - month_offset - 1).rem_euclid(12) + 1) as u32;
                    let date =
                        chrono::NaiveDate::from_ymd_opt(today.year(), target_month, 1).unwrap();
                    date.format("%b").to_string()
                })
                .collect();
            (bars, labels)
        }
    }
}

// ── Donut SVG ─────────────────────────────────────────────────────────────────

#[component]
fn DonutChart(mix: Vec<ActivityMixItem>) -> Element {
    let total: f32 = mix.iter().map(|a| a.hours).sum();
    if total <= 0.0 {
        return rsx! {};
    }

    let r = 50.0f32;
    let cx = 60.0f32;
    let cy = 60.0f32;
    let sw = 14.0f32;

    // Build arc paths
    let mut acc = 0.0f32;
    let arcs: Vec<(String, String)> = mix
        .iter()
        .map(|item| {
            let start_frac = acc / total;
            acc += item.hours;
            let end_frac = acc / total;

            let start_angle = start_frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let end_angle = end_frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

            let x1 = cx + r * start_angle.cos();
            let y1 = cy + r * start_angle.sin();
            let x2 = cx + r * end_angle.cos();
            let y2 = cy + r * end_angle.sin();
            let large_arc = if end_frac - start_frac > 0.5 { 1 } else { 0 };

            let d = format!("M {x1:.2} {y1:.2} A {r:.0} {r:.0} 0 {large_arc} 1 {x2:.2} {y2:.2}");
            (d, item.color.clone())
        })
        .collect();

    let total_str = fmt_hours(total);

    rsx! {
        div { class: "dash-donut-wrap",
            svg {
                view_box: "0 0 120 120",
                width: "140",
                height: "140",
                // Track ring
                circle {
                    cx: "{cx}",
                    cy: "{cy}",
                    r: "{r}",
                    fill: "none",
                    stroke: "var(--surface-2)",
                    stroke_width: "{sw}",
                }
                for (d, color) in arcs.iter() {
                    path {
                        d: "{d}",
                        stroke: "{color}",
                        stroke_width: "{sw}",
                        fill: "none",
                        stroke_linecap: "butt",
                    }
                }
                text {
                    x: "{cx}",
                    y: "{cy - 6.0}",
                    text_anchor: "middle",
                    font_size: "9",
                    fill: "var(--text-3)",
                    letter_spacing: "1.5",
                    "TOTAL"
                }
                text {
                    x: "{cx}",
                    y: "{cy + 11.0}",
                    text_anchor: "middle",
                    font_size: "15",
                    fill: "var(--color-foreground)",
                    font_family: "var(--font-mono)",
                    font_weight: "600",
                    "{total_str}"
                }
            }
            div { class: "dash-donut-legend",
                for item in mix.iter() {
                    div { class: "dash-donut-legend-row",
                        span {
                            class: "dash-donut-legend-dot",
                            style: "background:{item.color}",
                        }
                        span { class: "dash-donut-legend-name", "{item.name}" }
                        span { class: "dash-donut-legend-time", "{fmt_hours(item.hours)}" }
                        span { class: "dash-donut-legend-pct",
                            { format!("{:.0}%", item.hours / total * 100.0) }
                        }
                    }
                }
            }
        }
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn Dashboard() -> Element {
    let mut running: crate::RunningTimer = use_context();
    let mut toasts: Toasts = use_context();
    let user_settings: crate::UserSettings = use_context();

    let timesheets_cache: TimesheetsCache = use_context();
    let activities_cache: ActivitiesCache = use_context();

    let mut activities = use_signal(|| activities_cache.read().clone());
    let mut recent = use_signal(|| timesheets_cache.read().clone());

    let mut selected_activity_id = use_signal(|| Option::<String>::None);
    let elapsed_secs: crate::RunningElapsed = use_context();
    let mut chart_period = use_signal(|| ChartPeriod::Week);

    use_resource(move || async move {
        if let Ok(list) = api::activity::list_activities().await {
            activities.set(list);
        }
        if let Ok(list) = api::timesheet::list_timesheets().await {
            recent.set(list);
        }
    });

    let on_start = move |_| async move {
        let aid = selected_activity_id.peek().clone();
        match api::timesheet::start_timesheet(aid, None).await {
            Ok(dto) => {
                running.set(Some(dto));
                selected_activity_id.set(None);
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    };

    let on_stop = move |_| async move {
        let ts_id = running.peek().as_ref().map(|ts| ts.id.clone());
        if let Some(id) = ts_id {
            match api::timesheet::stop_timesheet(id).await {
                Ok(()) => {
                    running.set(None);
                    if let Ok(list) = api::timesheet::list_timesheets().await {
                        recent.set(list);
                    }
                }
                Err(e) => toasts.push_error(e.to_string()),
            }
        }
    };

    let stats = compute_stats(&recent.read(), &activities.read());
    let (chart_bars, chart_labels) =
        compute_chart_data(&recent.read(), &activities.read(), &chart_period.read());
    let has_data = chart_bars.iter().any(|segs| !segs.is_empty());

    let activity_colors: std::collections::HashMap<String, String> = activities
        .read()
        .iter()
        .map(|a| (a.id.clone(), a.color.clone()))
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        DefaultLayout {
            div { class: "space-y-6",

                // ── Quick Start / Running Timer ──────────────────────────────
                match running.read().clone() {
                    Some(ts) => {
                        let act_name = ts.activity_id.as_ref()
                            .and_then(|aid| activities.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
                            .unwrap_or_else(|| tid!("dashboard-unassigned"));
                        let e = *elapsed_secs.read();
                        rsx! {
                            Card { data_size: "md",
                                CardContent {
                                    div { class: "dashboard-timer",
                                        div { class: "dashboard-timer-header",
                                            div { class: "dashboard-timer-status",
                                                span { class: "timer-dot" }
                                                span { class: "dashboard-timer-status-label", {tid!("dashboard-timer-running")} }
                                            }
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                onclick: on_stop,
                                                Icon { icon: HiStop, width: 14, height: 14 }
                                                {tid!("common-stop")}
                                            }
                                        }
                                        div { class: "dashboard-timer-time",
                                            span { class: "dashboard-timer-elapsed",
                                                { format!("{:02}:{:02}:{:02}", e / 3600, (e % 3600) / 60, e % 60) }
                                            }
                                        }
                                        div { class: "dashboard-timer-meta",
                                            span { class: "text-sm font-medium", "{act_name}" }
                                            if let Some(ref desc) = ts.description {
                                                span { class: "text-xs text-secondary", "{desc}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    None => rsx! {
                        Card { data_size: "md",
                            CardContent {
                                div { class: "qs-card",
                                    div { class: "qs-header",
                                        span { class: "qs-label", {tid!("dashboard-quick-start")} }
                                        Icon { icon: HiLightningBolt, width: 14, height: 14 }
                                    }
                                    Select::<String> {
                                        options: activities.read().iter()
                                            .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                            .collect(),
                                        value: selected_activity_id.read().clone(),
                                        on_change: move |id: String| selected_activity_id.set(Some(id)),
                                        placeholder: tid!("common-select-activity"),
                                    }
                                    Button {
                                        onclick: on_start,
                                        class: "qs-start-btn",
                                        Icon { icon: HiPlay, width: 16, height: 16 }
                                        {tid!("dashboard-start-session")}
                                    }
                                }
                            }
                        }
                    },
                }

                // ── KPI Cards (4) ─────────────────────────────────────────────
                div { class: "dash-kpi-grid",
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-today")} }
                        span { class: "dash-kpi-value", "{fmt_hours(stats.today_hours)}" }
                        span { class: "dash-kpi-sub", {tid!("dashboard-tracked")} }
                    }
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-this-week")} }
                        span { class: "dash-kpi-value", "{fmt_hours(stats.week_hours)}" }
                        span { class: "dash-kpi-sub", {tid!("dashboard-tracked")} }
                    }
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-streak")} }
                        span { class: "dash-kpi-value", "{stats.streak}" }
                        span { class: "dash-kpi-sub", {tid!("dashboard-streak-unit")} }
                    }
                }

                // ── Charts ───────────────────────────────────────────────────
                if has_data {
                    div { class: "dash-charts-grid",

                        // Hours per day — bar chart with period toggle
                        div { class: "island dash-chart-island",
                            div { class: "island-header",
                                span { class: "island-title", {tid!("dashboard-hours-per-day")} }
                                div { class: "dash-period-tabs",
                                    button {
                                        class: if *chart_period.read() == ChartPeriod::Week { "dash-period-tab dash-period-tab--active" } else { "dash-period-tab" },
                                        onclick: move |_| chart_period.set(ChartPeriod::Week),
                                        {tid!("dashboard-chart-week")}
                                    }
                                    button {
                                        class: if *chart_period.read() == ChartPeriod::Month { "dash-period-tab dash-period-tab--active" } else { "dash-period-tab" },
                                        onclick: move |_| chart_period.set(ChartPeriod::Month),
                                        {tid!("dashboard-chart-month")}
                                    }
                                    button {
                                        class: if *chart_period.read() == ChartPeriod::Year { "dash-period-tab dash-period-tab--active" } else { "dash-period-tab" },
                                        onclick: move |_| chart_period.set(ChartPeriod::Year),
                                        {tid!("dashboard-chart-year")}
                                    }
                                }
                            }
                            div { class: "dash-chart-area",
                                HoursBarChart { bars: chart_bars, labels: chart_labels }
                            }
                        }

                        // Activity mix — donut chart
                        if !stats.activity_mix.is_empty() {
                            div { class: "island dash-chart-island",
                                div { class: "island-header",
                                    span { class: "island-title", {tid!("dashboard-activity-mix")} }
                                }
                                div { class: "dash-chart-area dash-chart-area--donut",
                                    DonutChart { mix: stats.activity_mix }
                                }
                            }
                        }
                    }
                }

                // Plugin-contributed dashboard widgets (§12.2 — dashboard.widgets).
                PluginSlot::<PluginHostCtx> { name: "dashboard.widgets".to_string() }

                // ── Recent Entries ───────────────────────────────────────────
                if !recent.read().is_empty() {
                    div { class: "island",
                        div { class: "island-header",
                            span { class: "island-title", {tid!("dashboard-recent-entries")} }
                        }
                        div { class: "flex flex-col",
                            for ts in recent.read().iter().take(6) {
                                {
                                    let ts = ts.clone();
                                    let act_name = ts.activity_id.as_ref()
                                        .and_then(|aid| activities.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
                                        .unwrap_or_else(|| "—".to_string());
                                    let act_color = ts.activity_id.as_ref()
                                        .and_then(|aid| activity_colors.get(aid).cloned())
                                        .unwrap_or_else(|| "var(--color-accent)".to_string());
                                    let duration_str = ts.duration.map(|d| {
                                        let h = d / 3600;
                                        let m = (d % 3600) / 60;
                                        if h > 0 { format!("{h}h {m:02}m") } else { format!("{m}m") }
                                    });
                                    let date_str = {
                                        let s = user_settings.read();
                                        formatting::format_datetime(&ts.start_time, &s.timezone, &s.date_format)
                                    };
                                    rsx! {
                                        div {
                                            key: "{ts.id}",
                                            class: "dash-entry",
                                            div {
                                                class: "dash-entry-color",
                                                style: "background:{act_color}",
                                            }
                                            div { class: "dash-entry-main",
                                                div { class: "dash-entry-name",
                                                    if let Some(ref desc) = ts.description {
                                                        "{desc}"
                                                    } else {
                                                        "{act_name}"
                                                    }
                                                }
                                                div { class: "dash-entry-meta",
                                                    span { "{act_name}" }
                                                    span { class: "dash-entry-sep", "·" }
                                                    span { "{date_str}" }
                                                    for tag in ts.tags.iter() {
                                                        span { class: "dash-tag-pill", "#{tag.name}" }
                                                    }
                                                }
                                            }
                                            div { class: "dash-entry-time",
                                                if let Some(ref d) = duration_str {
                                                    "{d}"
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
    }
}
