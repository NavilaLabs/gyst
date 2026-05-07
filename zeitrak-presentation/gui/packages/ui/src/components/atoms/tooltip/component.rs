use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::tooltip::{self, TooltipContentProps, TooltipProps, TooltipTriggerProps};

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let base = attributes!(div { class: "tooltip" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        tooltip::Tooltip {
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let base = attributes!(div {
        class: "tooltip-trigger"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        tooltip::TooltipTrigger { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let base = attributes!(div {
        class: "tooltip-content"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        tooltip::TooltipContent {
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: merged,
            {props.children}
        }
    }
}
