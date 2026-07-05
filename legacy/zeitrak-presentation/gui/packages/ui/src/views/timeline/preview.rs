use dioxus::prelude::*;

use super::{
    rail::{GapSegment, RailSegment, TimelineRailColumn},
    station::{StationEntry, TimelineStation},
};

// Fake timeline at 1.0 px/min scale, t_max = 18:30
//
// Station layout (newest at top):
//   s1  Evening Run   running  17:18 →        dot_y=72   seg top=0   h=72   #f59e0b
//   s2  Code Review   14:00–16:30 2h30m        dot_y=270  seg top=120 h=150  #4f46e5
//   s3  Lunch         12:45–13:30 45m          dot_y=345  seg top=300 h=45   #059669
//   s4  Deep Work     09:15–12:30 3h15m        dot_y=555  seg top=360 h=195  #4f46e5
//
// Gaps: (72,48)  (270,30)  (345,15)  tail(555,∞)
// Schwellen every 60 min: y = 30, 90, 150, 210, 270, 330, 390, 450, 510, 570

/// Static decorative preview of the timeline, used on the login page.
///
/// All data is hardcoded — no API calls, no signals.
#[component]
pub fn TimelinePreview() -> Element {
    let segments = vec![
        RailSegment { top: 0.0,   height: 72.0,  color: "#f59e0b".to_string() },
        RailSegment { top: 120.0, height: 150.0, color: "#4f46e5".to_string() },
        RailSegment { top: 300.0, height: 45.0,  color: "#059669".to_string() },
        RailSegment { top: 360.0, height: 195.0, color: "#4f46e5".to_string() },
    ];
    let gaps = vec![
        GapSegment { top: 72.0,  height: 48.0  },
        GapSegment { top: 270.0, height: 30.0  },
        GapSegment { top: 345.0, height: 15.0  },
        GapSegment { top: 555.0, height: 800.0 },
    ];
    let schwellen: Vec<f64> = vec![30.0, 90.0, 150.0, 210.0, 270.0, 330.0, 390.0, 450.0, 510.0, 570.0];

    rsx! {
        div { class: "tl-preview-wrap",
            div {
                class: "tl-inner",
                style: "min-height:max(680px,100%)",
                TimelineRailColumn { schwellen, segments, gaps }
                TimelineStation {
                    key: "prev-s1",
                    dot_y: 72.0,
                    side: "left".to_string(),
                    is_running: true,
                    dot_color: "#f59e0b".to_string(),
                    title: "Evening Run".to_string(),
                    entries: vec![StationEntry {
                        activity_name: Some("Evening Run".to_string()),
                        activity_color: Some("#f59e0b".to_string()),
                        time_range: "17:18 –".to_string(),
                        duration: Some("1h 12m".to_string()),
                        description: None,
                        is_running: true,
                    }],
                    total_duration: "1h 12m".to_string(),
                }
                TimelineStation {
                    key: "prev-s2",
                    dot_y: 270.0,
                    side: "right".to_string(),
                    is_running: false,
                    dot_color: "#4f46e5".to_string(),
                    title: "Code Review".to_string(),
                    entries: vec![StationEntry {
                        activity_name: Some("Code Review".to_string()),
                        activity_color: Some("#4f46e5".to_string()),
                        time_range: "14:00 – 16:30".to_string(),
                        duration: Some("2h 30m".to_string()),
                        description: None,
                        is_running: false,
                    }],
                    total_duration: "2h 30m".to_string(),
                }
                TimelineStation {
                    key: "prev-s3",
                    dot_y: 345.0,
                    side: "left".to_string(),
                    is_running: false,
                    dot_color: "#059669".to_string(),
                    title: "Lunch".to_string(),
                    entries: vec![StationEntry {
                        activity_name: Some("Lunch".to_string()),
                        activity_color: Some("#059669".to_string()),
                        time_range: "12:45 – 13:30".to_string(),
                        duration: Some("45m".to_string()),
                        description: None,
                        is_running: false,
                    }],
                    total_duration: "45m".to_string(),
                }
                TimelineStation {
                    key: "prev-s4",
                    dot_y: 555.0,
                    side: "right".to_string(),
                    is_running: false,
                    dot_color: "#4f46e5".to_string(),
                    title: "Deep Work".to_string(),
                    entries: vec![StationEntry {
                        activity_name: Some("Deep Work".to_string()),
                        activity_color: Some("#4f46e5".to_string()),
                        time_range: "09:15 – 12:30".to_string(),
                        duration: Some("3h 15m".to_string()),
                        description: None,
                        is_running: false,
                    }],
                    total_duration: "3h 15m".to_string(),
                }
            }
            div { class: "tl-preview-fade" }
        }
    }
}
