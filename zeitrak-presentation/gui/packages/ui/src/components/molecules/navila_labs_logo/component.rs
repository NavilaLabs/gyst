use dioxus::prelude::*;

#[component]
pub fn NavilaLabsLogo(
    #[props(extends=GlobalAttributes)]
    #[props(extends=svg)]
    attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 420 500",
            "xmlns:bx": "https://boxy-svg.com",
            ..attributes,
            text {
                style: "fill: var(--color-text-primary); font-family: Bitcount; font-size: 830px; font-weight: 380; paint-order: fill; text-anchor: start;",
                x: "0",
                y: "500",
                "N"
            }
        }
    }
}
