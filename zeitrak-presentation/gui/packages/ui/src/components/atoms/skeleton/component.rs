use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// A single shimmering rectangle — the base building block for all skeletons.
///
/// Size and shape are controlled via `class` or inline style overrides.
/// ```rust,ignore
/// Skeleton { class: "h-4 w-32 rounded" }
/// ```
#[component]
pub fn Skeleton(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    let base = attributes!(div { class: "skeleton" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { ..merged }
    }
}

/// A skeleton that mimics a typical list-item card.
///
/// Renders a card-shaped container with three shimmer lines (title, subtitle,
/// metadata). Drop N of these into a `div.flex.flex-col.gap-3` while the real
/// data is loading.
#[component]
pub fn SkeletonListItem() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "skeleton-list-item",
            div { class: "skeleton skeleton-line-title" }
            div { class: "skeleton skeleton-line-sub" }
            div { class: "skeleton skeleton-line-meta" }
        }
    }
}
