use dioxus::prelude::*;

use crate::components::atoms::card::{Card, CardContent, CardHeader, CardTitle};

/// A single timesheet entry rendered inside a station card.
#[derive(Clone, PartialEq, Debug)]
pub struct StationEntry {
    pub activity_name: Option<String>,
    pub activity_color: Option<String>,
    pub time_range: String,
    pub duration: Option<String>,
    pub description: Option<String>,
    pub is_running: bool,
}

/// A positioned point on the timeline: dot on the rail, connector line, and an info card.
#[component]
pub fn TimelineStation(
    dot_y: f64,
    side: String,
    is_running: bool,
    dot_color: String,
    title: String,
    entries: Vec<StationEntry>,
    total_duration: String,
) -> Element {
    let multi = entries.len() > 1;
    rsx! {
        div {
            class: "tl-station",
            "data-side": "{side}",
            style: "top:{dot_y:.1}px",
            div {
                class: if is_running { "tl-dot tl-dot--running" } else { "tl-dot" },
                style: "--dot-color:{dot_color}",
            }
            div { class: "tl-card-wrap",
                div { class: "tl-connector" }
                Card {
                    CardHeader {
                        CardTitle {
                            if !multi {
                                if let Some(color) = entries.first().and_then(|e| e.activity_color.as_deref()) {
                                    span { class: "zk-activity-dot", style: "background:{color}" }
                                }
                            }
                            "{title}"
                        }
                    }
                    CardContent {
                        div { class: "tl-card-body",
                            for entry in &entries {
                                div { class: "tl-card-entry",
                                    if multi {
                                        if let (Some(name), Some(color)) = (
                                            entry.activity_name.as_deref(),
                                            entry.activity_color.as_deref(),
                                        ) {
                                            span { class: "zk-activity-dot", style: "background:{color}" }
                                            span { class: "tl-act-label", "{name}" }
                                        }
                                    }
                                    div { class: "tl-entry-meta",
                                        span { class: "tl-entry-time font-mono", "{entry.time_range}" }
                                        if let Some(dur) = &entry.duration {
                                            span {
                                                class: if entry.is_running {
                                                    "tl-entry-dur tl-entry-dur--live font-mono"
                                                } else {
                                                    "tl-entry-dur font-mono"
                                                },
                                                "{dur}"
                                            }
                                        }
                                    }
                                    if let Some(desc) = &entry.description {
                                        if !desc.is_empty() {
                                            p { class: "tl-entry-notes", "{desc}" }
                                        }
                                    }
                                }
                            }
                            div { class: "tl-card-total",
                                if is_running { span { class: "timer-dot" } }
                                span { class: "font-mono", "{total_duration}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
