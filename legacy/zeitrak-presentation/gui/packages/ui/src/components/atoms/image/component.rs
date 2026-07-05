use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[component]
pub fn Image(
    src: String,
    alt: String,
    width: Option<i64>,
    height: Option<i64>,
    #[props(extends=GlobalAttributes)]
    #[props(extends=img)]
    attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(img { loading: "lazy" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        img { src, alt, width, height, ..merged }
    }
}
