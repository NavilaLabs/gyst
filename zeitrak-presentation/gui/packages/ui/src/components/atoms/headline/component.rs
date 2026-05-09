use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[component]
pub fn Headline(
    #[props(extends=GlobalAttributes)]
    #[props(extends=h2)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(h2 { class: "headline" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        h2 { ..merged, {children} }
    }
}
