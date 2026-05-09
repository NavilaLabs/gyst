use dioxus::prelude::*;

use crate::components::atoms::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};

#[derive(Clone, PartialEq, Debug)]
pub struct SelectOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
    /// Optional muted subtitle shown below the label in the dropdown.
    pub sublabel: Option<String>,
}

impl<T: Clone + PartialEq> SelectOption<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            sublabel: None,
        }
    }

    pub fn sublabel(mut self, s: impl Into<String>) -> Self {
        self.sublabel = Some(s.into());
        self
    }
}

/// A single-value select backed by our `DropdownMenu` primitive.
///
/// ```rust
/// Select::<String> {
///     options: vec![SelectOption::new("a".to_string(), "Option A")],
///     value: selected.read().clone(),
///     on_change: move |v| selected.set(v),
///     placeholder: "Choose…".to_string(),
/// }
/// ```
#[component]
pub fn Select<T: Clone + PartialEq + 'static>(
    options: Vec<SelectOption<T>>,
    value: Option<T>,
    on_change: EventHandler<T>,
    #[props(default = "Select…".to_string())] placeholder: String,
) -> Element {
    let selected_label = options
        .iter()
        .find(|o| value.as_ref().is_some_and(|v| v == &o.value))
        .map(|o| o.label.clone())
        .unwrap_or_else(|| placeholder.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "select",
            DropdownMenu {
                DropdownMenuTrigger {
                    span { class: "select-label", "{selected_label}" }
                    span { class: "select-chevron", "▾" }
                }
                DropdownMenuContent {
                    for (i, option) in options.into_iter().enumerate() {
                        {
                            let val = option.value;
                            let label = option.label.clone();
                            let sublabel = option.sublabel.clone();
                            rsx! {
                                DropdownMenuItem {
                                    key: "{label}",
                                    value: val,
                                    index: i,
                                    on_select: move |v: T| on_change.call(v),
                                    if let Some(ref sub) = sublabel {
                                        div { class: "select-option-group",
                                            span { "{label}" }
                                            span { class: "select-option-sublabel", "{sub}" }
                                        }
                                    } else {
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
