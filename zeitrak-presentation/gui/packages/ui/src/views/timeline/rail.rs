use dioxus::prelude::*;

/// A single colored activity segment on the timeline rail.
#[derive(Clone, PartialEq, Debug)]
pub struct RailSegment {
    pub top: f64,
    pub height: f64,
    pub color: String,
}

/// A gray gap between activity segments on the timeline rail.
#[derive(Clone, PartialEq, Debug)]
pub struct GapSegment {
    pub top: f64,
    pub height: f64,
}

/// The vertical center rail: colored segments, gray gaps, and time-interval cross-ties.
#[component]
pub fn TimelineRailColumn(
    schwellen: Vec<f64>,
    segments: Vec<RailSegment>,
    gaps: Vec<GapSegment>,
) -> Element {
    rsx! {
        div { class: "tl-rail-col",
            for y in &schwellen {
                div { class: "tl-schwelle", style: "top:{y:.1}px" }
            }
            for seg in &segments {
                div {
                    class: "tl-segment",
                    style: "top:{seg.top:.1}px;height:{seg.height:.1}px;--seg-color:{seg.color}",
                }
            }
            for gap in &gaps {
                div {
                    class: "tl-gap",
                    style: "top:{gap.top:.1}px;height:{gap.height:.1}px",
                }
            }
        }
    }
}
