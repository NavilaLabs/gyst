use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::navbar::{
    self, NavbarContentProps, NavbarItemProps, NavbarNavProps, NavbarProps, NavbarTriggerProps,
};

#[component]
pub fn Navbar(props: NavbarProps) -> Element {
    let base = attributes!(div { class: "navbar" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        navbar::Navbar {
            disabled: props.disabled,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn NavbarNav(props: NavbarNavProps) -> Element {
    let base = attributes!(div {
        class: "navbar-nav"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        navbar::NavbarNav {
            index: props.index,
            disabled: props.disabled,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn NavbarTrigger(props: NavbarTriggerProps) -> Element {
    let base = attributes!(button {
        class: "navbar-trigger"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        navbar::NavbarTrigger {
            attributes: merged,
            {props.children}
            svg {
                class: "navbar-expand-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[component]
pub fn NavbarContent(props: NavbarContentProps) -> Element {
    let base = attributes!(div {
        class: "navbar-content"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        navbar::NavbarContent { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    let base = attributes!(a {
        class: "navbar-item"
    });
    let user_class = props
        .class
        .as_deref()
        .filter(|c| !c.is_empty())
        .map(|c| attributes!(a { class: c }))
        .unwrap_or_default();
    let merged = merge_attributes(vec![base, user_class, props.attributes.clone()]);

    rsx! {
        navbar::NavbarItem {
            index: props.index,
            value: props.value,
            disabled: props.disabled,
            new_tab: props.new_tab,
            to: props.to,
            active_class: props.active_class,
            attributes: merged,
            on_select: props.on_select,
            onclick: props.onclick,
            onmounted: props.onmounted,
            {props.children}
        }
    }
}
