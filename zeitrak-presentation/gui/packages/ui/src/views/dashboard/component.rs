use crate::components::atoms::card::{Card, CardContent};
use crate::components::atoms::{Button, ButtonVariant, Select, SelectOption, ToastExt, Toasts};
use crate::formatting;
use crate::layouts::DefaultLayout;
use crate::{ActivitiesCache, TimesheetsCache};
use chrono::{Datelike, Duration, Utc};
use dioxus::prelude::*;
use dioxus_charts::BarChart;
use dioxus_free_icons::icons::hi_solid_icons::{HiLightningBolt, HiPlay, HiStop};
use dioxus_free_icons::Icon;

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

struct DashStats {
    today_hours: f32,
    week_hours: f32,
    hours_by_day: Vec<f32>,
    day_labels: Vec<String>,
    has_week_data: bool,
}

fn compute_stats(timesheets: &[api::timesheet::TimesheetDto]) -> DashStats {
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

    let hours_by_day: Vec<f32> = (0..7)
        .map(|i| {
            let day = today - Duration::days(6 - i as i64);
            timesheets
                .iter()
                .filter(|ts| ts.duration.is_some() && parse_date(&ts.start_time) == Some(day))
                .map(|ts| ts.duration.unwrap_or(0) as f32 / 3600.0)
                .sum::<f32>()
        })
        .collect();

    let day_labels: Vec<String> = (0..7)
        .map(|i| {
            let day = today - Duration::days(6 - i as i64);
            day.format("%a").to_string()
        })
        .collect();

    let has_week_data = week_hours > 0.0;

    DashStats {
        today_hours,
        week_hours,
        hours_by_day,
        day_labels,
        has_week_data,
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

    // Compute all stats before entering rsx! (drops the borrow immediately).
    let stats = compute_stats(&recent.read());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        DefaultLayout {
            div { class: "space-y-6",

                // ── Quick Start / Running Timer ──────────────────────────────
                match running.read().clone() {
                    Some(ts) => {
                        let act_name = ts.activity_id.as_ref()
                            .and_then(|aid| activities.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
                            .unwrap_or_else(|| "Unassigned".to_string());
                        let e = *elapsed_secs.read();
                        rsx! {
                            Card { data_size: "md",
                                CardContent {
                                    div { class: "dashboard-timer",
                                        div { class: "dashboard-timer-header",
                                            div { class: "dashboard-timer-status",
                                                span { class: "timer-dot" }
                                                span { class: "dashboard-timer-status-label", "Timer Running" }
                                            }
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                onclick: on_stop,
                                                Icon { icon: HiStop, width: 14, height: 14 }
                                                "Stop"
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
                                        span { class: "qs-label", "Quick Start" }
                                        Icon { icon: HiLightningBolt, width: 14, height: 14 }
                                    }
                                    Select::<String> {
                                        options: activities.read().iter()
                                            .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                            .collect(),
                                        value: selected_activity_id.read().clone(),
                                        on_change: move |id: String| selected_activity_id.set(Some(id)),
                                        placeholder: "Select activity…".to_string(),
                                    }
                                    Button {
                                        onclick: on_start,
                                        class: "qs-start-btn",
                                        Icon { icon: HiPlay, width: 16, height: 16 }
                                        "Start Session"
                                    }
                                }
                            }
                        }
                    },
                }

                // ── KPI Cards ────────────────────────────────────────────────
                div { class: "dash-kpi-grid",
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", "Today" }
                        span { class: "dash-kpi-value", "{fmt_hours(stats.today_hours)}" }
                        span { class: "dash-kpi-sub", "tracked" }
                    }
                    div { class: "dash-kpi-card",
                        span { class: "dash-kpi-label", "This Week" }
                        span { class: "dash-kpi-value", "{fmt_hours(stats.week_hours)}" }
                        span { class: "dash-kpi-sub", "tracked" }
                    }
                }

                // ── Charts ───────────────────────────────────────────────────
                if stats.has_week_data {
                    div { class: "dash-charts-grid",

                        // Hours per day — bar chart
                        div { class: "island dash-chart-island",
                            div { class: "island-header",
                                span { class: "island-title", "Hours per Day" }
                                span { class: "island-subtitle", "last 7 days" }
                            }
                            div { class: "dash-chart-area",
                                BarChart {
                                    series: vec![stats.hours_by_day],
                                    labels: Some(stats.day_labels),
                                    padding_top: 20,
                                    padding_bottom: 36,
                                    padding_left: 50,
                                    padding_right: 10,
                                    label_interpolation: Some(fmt_hours_axis as fn(f32) -> String),
                                    label_size: 25,
                                    show_dotted_grid: false,
                                    show_series_labels: false,
                                    bar_width: "45",
                                    bar_distance: 0.0,
                                    width: "100%",
                                    height: "100%",
                                    viewbox_width: 560,
                                    viewbox_height: 240,
                                    class_chart_bar: "dx-chart-bar",
                                    class_bar: "dx-bar",
                                    class_bar_group: "dx-bar-group",
                                    class_bar_label: "dx-bar-label",
                                    class_grid: "dx-grid",
                                    class_grid_line: "dx-grid-line",
                                    class_grid_label: "dx-grid-label",
                                    class_grid_labels: "dx-grid-labels",
                                }
                            }
                        }
                    }
                }

                // ── Recent Entries ───────────────────────────────────────────
                if !recent.read().is_empty() {
                    div { class: "island",
                        div { class: "island-header",
                            span { class: "island-title", "Recent Entries" }
                        }
                        div { class: "flex flex-col gap-2",
                            for ts in recent.read().iter().take(5) {
                                {
                                    let act_name = ts.activity_id.as_ref()
                                        .and_then(|aid| activities.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
                                        .unwrap_or_else(|| "—".to_string());
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
                                        Card { key: "{ts.id}",
                                            CardContent {
                                                div { class: "flex items-center justify-between",
                                                    div { class: "flex flex-col gap-1",
                                                        span { class: "font-medium text-sm",
                                                            if let Some(ref desc) = ts.description {
                                                                "{desc}"
                                                            } else {
                                                                "{act_name}"
                                                            }
                                                        }
                                                        span { class: "text-xs text-secondary",
                                                            "{act_name}"
                                                        }
                                                        span { class: "text-xs text-secondary", "{date_str}" }
                                                    }
                                                    div { class: "flex flex-col items-end gap-1",
                                                        if let Some(ref d) = duration_str {
                                                            span { class: "text-sm font-medium", "{d}" }
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
    }
}
