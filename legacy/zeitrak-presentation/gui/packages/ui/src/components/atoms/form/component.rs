use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[component]
pub fn Form(
    #[props(extends=GlobalAttributes)]
    #[props(extends=form)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(form {
        class: "form w-full"
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        form { ..merged, {children} }
    }
}

#[component]
pub fn FormField(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "form-field w-full my-2"
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        div { ..merged, {children} }
    }
}
