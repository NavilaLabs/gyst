use dioxus::prelude::*;

#[component]
pub fn ZeitrakLogo(
    #[props(extends=GlobalAttributes)]
    #[props(extends=svg)]
    attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 2010 710",
            "xmlns:bx": "https://boxy-svg.com",
            ..attributes,
            text {
                style: "fill: var(--color-text-primary); font-size: 830px; font-family: 'Pacifico'; font-weight: 380; paint-order: fill; text-anchor: start;",
                x: "50",
                y: "705",
                "Zeitrak"
            }
        }
    }
}
