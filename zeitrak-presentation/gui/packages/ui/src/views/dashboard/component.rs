use crate::components::atoms::card::{Card, CardContent};
use crate::components::atoms::{
    Button, ButtonVariant, Select, SelectOption, Skeleton, ToastExt, Toasts,
};
use crate::components::molecules::MemberFilter;
use crate::layouts::DefaultLayout;
use crate::ActivitiesCache;
// use crate::PluginHostCtx;
use dioxus::prelude::*;
// use dioxus_extism_frontend::PluginSlot;
use dioxus_free_icons::icons::hi_solid_icons::{HiLightningBolt, HiPlay, HiStop};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

// ── Chart period ──────────────────────────────────────────────────────────────

use api::timesheet::DashboardPeriod;

#[derive(Clone, PartialEq)]
enum ChartPeriod {
    Week,
    Month,
    Year,
}

impl From<ChartPeriod> for DashboardPeriod {
    fn from(p: ChartPeriod) -> Self {
        match p {
            ChartPeriod::Week => DashboardPeriod::Week,
            ChartPeriod::Month => DashboardPeriod::Month,
            ChartPeriod::Year => DashboardPeriod::Year,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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

// ── Rendering types (data comes from server via DashboardStatsDto) ────────────

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

// ── Activity mix bar ──────────────────────────────────────────────────────────

/// Segments with a share below this threshold are collapsed into "Others".
const MIX_BAR_MIN_PCT: f32 = 7.0;

#[component]
fn ActivityMixBar(mix: Vec<api::timesheet::ActivityMixPoint>) -> Element {
    let mut show_tooltip = use_signal(|| false);
    let others_label = tid!("dashboard-mix-others");

    let total: f32 = mix.iter().map(|m| m.hours).sum();
    if total <= 0.0 {
        return rsx! {
            span { class: "mix-bar-empty", "—" }
        };
    }

    let others_hours: f32 = mix
        .iter()
        .filter(|m| m.percentage < MIX_BAR_MIN_PCT)
        .map(|m| m.hours)
        .sum();

    // Segments rendered in the bar (big ones first, then "others" at the end).
    let bar_segs: Vec<(String, String, f32)> = {
        let mut segs: Vec<(String, String, f32)> = mix
            .iter()
            .filter(|m| m.percentage >= MIX_BAR_MIN_PCT)
            .map(|m| (m.activity_name.clone(), m.color.clone(), m.hours))
            .collect();
        if others_hours > 0.01 {
            segs.push((others_label.clone(), "#6c6c76".to_string(), others_hours));
        }
        segs
    };

    rsx! {
        div {
            class: "mix-bar-wrap",
            onmouseenter: move |_| show_tooltip.set(true),
            onmouseleave: move |_| show_tooltip.set(false),

            div { class: "mix-bar",
                for (_, color, hours) in bar_segs.iter() {
                    {
                        let flex_val = format!("{:.4}", hours / total * 100.0);
                        let style = format!("flex: {flex_val}; background: {color}");
                        rsx! {
                            div { class: "mix-bar-seg", style: "{style}" }
                        }
                    }
                }
            }

            if *show_tooltip.read() {
                div { class: "mix-bar-tooltip",
                    for item in mix.iter() {
                        div { class: "mix-bar-tooltip-row",
                            span {
                                class: "mix-bar-tooltip-dot",
                                style: "background: {item.color}",
                            }
                            span { class: "mix-bar-tooltip-name", "{item.activity_name}" }
                            span { class: "mix-bar-tooltip-hours", "{fmt_hours(item.hours)}" }
                            span { class: "mix-bar-tooltip-pct",
                                { format!("{:.0}%", item.percentage) }
                            }
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
    let activities_cache: ActivitiesCache = use_context();

    let mut selected_activity_id = use_signal(|| Option::<String>::None);
    let elapsed_secs: crate::RunningElapsed = use_context();
    let mut chart_period = use_signal(|| ChartPeriod::Week);
    let mut member_id: Signal<Option<String>> = use_signal(|| None);
    let mut can_filter_members = use_signal(|| false);
    let mut members: Signal<Vec<api::member::MemberDto>> = use_signal(Vec::new);
    let mut stats: Signal<Option<api::timesheet::DashboardStatsDto>> = use_signal(|| None);
    let mut loading = use_signal(|| true);

    let mut monthly_rows: Signal<Vec<api::timesheet::MonthlyOverviewRow>> =
        use_signal(Vec::new);
    let mut monthly_loading = use_signal(|| true);

    use_resource(move || async move {
        loading.set(true);
        let mid = member_id.read().clone();
        let period: api::timesheet::DashboardPeriod = chart_period.read().clone().into();
        if let Ok(s) = api::timesheet::dashboard_stats(mid, period).await {
            // peek() avoids a reactive subscription that would restart this resource
            // when members.set(list) fires below.
            if s.can_filter_members && members.peek().is_empty() {
                if let Ok(list) = api::member::list_members().await {
                    members.set(list);
                }
            }
            can_filter_members.set(s.can_filter_members);
            stats.set(Some(s));
        }
        loading.set(false);
    });

    use_resource(move || async move {
        monthly_loading.set(true);
        let mid = member_id.read().clone();
        if let Ok(rows) = api::timesheet::monthly_overview(mid).await {
            monthly_rows.set(rows);
        }
        monthly_loading.set(false);
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
                    // Re-fetch dashboard stats after stop so charts update.
                    let mid = member_id.peek().clone();
                    let period: api::timesheet::DashboardPeriod =
                        chart_period.peek().clone().into();
                    if let Ok(s) = api::timesheet::dashboard_stats(mid.clone(), period).await {
                        stats.set(Some(s));
                    }
                    if let Ok(rows) = api::timesheet::monthly_overview(mid).await {
                        monthly_rows.set(rows);
                    }
                }
                Err(e) => toasts.push_error(e.to_string()),
            }
        }
    };

    let on_member_change = move |new_member: Option<String>| {
        member_id.set(new_member);
    };

    // Derive rendering data from server stats.
    let today_hours = stats.read().as_ref().map(|s| s.today_hours).unwrap_or(0.0);
    let week_hours = stats.read().as_ref().map(|s| s.week_hours).unwrap_or(0.0);
    let streak = stats.read().as_ref().map(|s| s.streak).unwrap_or(0);

    let chart_bars: Vec<Vec<BarSegment>> = stats
        .read()
        .as_ref()
        .map(|s| {
            s.chart_bars
                .iter()
                .map(|pt| {
                    pt.segments
                        .iter()
                        .map(|(_, name, color, hours)| BarSegment {
                            name: name.clone(),
                            color: color.clone(),
                            hours: *hours,
                        })
                        .collect()
                })
                .collect()
        })
        .unwrap_or_default();
    let chart_labels: Vec<String> = stats
        .read()
        .as_ref()
        .map(|s| s.chart_bars.iter().map(|pt| pt.label.clone()).collect())
        .unwrap_or_default();
    let has_data = chart_bars.iter().any(|segs| !segs.is_empty());

    let mix: Vec<ActivityMixItem> = stats
        .read()
        .as_ref()
        .map(|s| {
            s.activity_mix
                .iter()
                .map(|am| ActivityMixItem {
                    name: am.activity_name.clone(),
                    color: am.color.clone(),
                    hours: am.hours,
                })
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        DefaultLayout {
            div { class: "space-y-6",

                // ── Member filter (workspace admins only) ────────────────────
                if *can_filter_members.read() {
                    MemberFilter {
                        members,
                        selected: member_id,
                        on_change: on_member_change,
                    }
                }

                // ── Quick Start / Running Timer ──────────────────────────────
                match running.read().clone() {
                    Some(ts) => {
                        let act_name = ts.activity_id.as_ref()
                            .and_then(|aid| activities_cache.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
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
                                        options: activities_cache.read().iter()
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

                // ── KPI Cards ─────────────────────────────────────────────────
                div { class: "dash-kpi-grid",
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-today")} }
                        if *loading.read() {
                            Skeleton { class: "h-7 w-14 rounded" }
                        } else {
                            span { class: "dash-kpi-value", "{fmt_hours(today_hours)}" }
                        }
                        span { class: "dash-kpi-sub", {tid!("dashboard-tracked")} }
                    }
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-this-week")} }
                        if *loading.read() {
                            Skeleton { class: "h-7 w-14 rounded" }
                        } else {
                            span { class: "dash-kpi-value", "{fmt_hours(week_hours)}" }
                        }
                        span { class: "dash-kpi-sub", {tid!("dashboard-tracked")} }
                    }
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", {tid!("dashboard-streak")} }
                        if *loading.read() {
                            Skeleton { class: "h-7 w-10 rounded" }
                        } else {
                            span { class: "dash-kpi-value", "{streak}" }
                        }
                        span { class: "dash-kpi-sub", {tid!("dashboard-streak-unit")} }
                    }
                }

                // ── Charts ───────────────────────────────────────────────────
                if *loading.read() {
                    div { class: "dash-charts-grid",
                        div { class: "island dash-chart-island",
                            div { class: "island-header",
                                Skeleton { class: "h-4 w-32 rounded" }
                            }
                            div { class: "dash-chart-area",
                                Skeleton { class: "w-full h-full rounded" }
                            }
                        }
                    }
                } else if has_data {
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
                        if !mix.is_empty() {
                            div { class: "island dash-chart-island",
                                div { class: "island-header",
                                    span { class: "island-title", {tid!("dashboard-activity-mix")} }
                                }
                                div { class: "dash-chart-area dash-chart-area--donut",
                                    DonutChart { mix: mix.clone() }
                                }
                            }
                        }
                    }
                }

                // PluginSlot::<PluginHostCtx> { name: "dashboard.widgets".to_string() }

                // ── Monthly Overview ─────────────────────────────────────────
                div { class: "island",
                    div { class: "island-header",
                        span { class: "island-title", {tid!("dashboard-monthly-overview")} }
                    }
                    if *monthly_loading.read() {
                        div { class: "month-table-skeleton",
                            for _ in 0..4 {
                                div { class: "month-table-skeleton-row",
                                    Skeleton { class: "h-4 w-20 rounded" }
                                    Skeleton { class: "h-4 w-10 rounded" }
                                    Skeleton { class: "h-4 w-full rounded" }
                                    Skeleton { class: "h-4 w-14 rounded" }
                                }
                            }
                        }
                    } else if monthly_rows.read().is_empty() {
                        div { class: "month-table-empty",
                            {tid!("dashboard-monthly-empty")}
                        }
                    } else {
                        table { class: "month-table",
                            thead {
                                tr {
                                    th { {tid!("dashboard-month-col")} }
                                    th { {tid!("dashboard-year-col")} }
                                    th { {tid!("dashboard-mix-col")} }
                                    th { class: "month-table-th-total", {tid!("dashboard-total-col")} }
                                }
                            }
                            tbody {
                                for row in monthly_rows.read().iter() {
                                    {
                                        let row = row.clone();
                                        let total_str = fmt_hours(row.total_hours);
                                        rsx! {
                                            tr { key: "{row.year}-{row.month}",
                                                td { class: "month-table-month",
                                                    { match row.month {
                                                        1 => tid!("dashboard-month-1"),
                                                        2 => tid!("dashboard-month-2"),
                                                        3 => tid!("dashboard-month-3"),
                                                        4 => tid!("dashboard-month-4"),
                                                        5 => tid!("dashboard-month-5"),
                                                        6 => tid!("dashboard-month-6"),
                                                        7 => tid!("dashboard-month-7"),
                                                        8 => tid!("dashboard-month-8"),
                                                        9 => tid!("dashboard-month-9"),
                                                        10 => tid!("dashboard-month-10"),
                                                        11 => tid!("dashboard-month-11"),
                                                        12 => tid!("dashboard-month-12"),
                                                        _ => String::new(),
                                                    }}
                                                }
                                                td { class: "month-table-year",
                                                    { row.year.to_string() }
                                                }
                                                td { class: "month-table-mix",
                                                    ActivityMixBar { mix: row.activity_mix.clone() }
                                                }
                                                td { class: "month-table-total",
                                                    "{total_str}"
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
