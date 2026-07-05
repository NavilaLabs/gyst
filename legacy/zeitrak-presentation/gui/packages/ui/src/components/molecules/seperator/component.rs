use dioxus::prelude::*;

#[component]
pub fn Seperator(
    width: Option<String>,
    #[props(extends=GlobalAttributes)]
    #[props(extends=svg)]
    attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        svg {
            class: "seperator h-16",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 160 500",
            "xmlns:bx": "https://boxy-svg.com",
            rect {
                width: width.unwrap_or("20".to_string()),
                height: "500",
                x: "75",
                y: "0",
                style: "fill: var(--color-text-primary);",
            }
        }
    }
}
