use crate::components::atoms::select::SelectOption;
use dioxus::prelude::*;

/// A single-value select with a search input and a scrollable, filtered list.
///
/// Renders a trigger button that shows the selected label (or a placeholder).
/// Clicking the trigger opens a panel with a text input at the top and a
/// scrollable list of matching options below.  Clicking outside the panel or
/// selecting an option closes it.
///
/// ```rust
/// SearchableSelect::<String> {
///     options: timezone_options(),
///     value: Some(user_timezone.read().clone()),
///     on_change: move |v| user_timezone.set(v),
///     placeholder: "Select timezone".to_string(),
/// }
/// ```
#[component]
pub fn SearchableSelect<T: Clone + PartialEq + std::fmt::Debug + 'static>(
    options: Vec<SelectOption<T>>,
    value: Option<T>,
    on_change: EventHandler<T>,
    #[props(default = "Select…".to_string())] placeholder: String,
) -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);

    let selected_label = options
        .iter()
        .find(|o| value.as_ref().is_some_and(|v| v == &o.value))
        .map(|o| o.label.clone())
        .unwrap_or_else(|| placeholder.clone());

    // Filtered list — computed from query signal each render.
    let q = query.read().to_lowercase();
    let filtered: Vec<SelectOption<T>> = options
        .iter()
        .filter(|o| o.label.to_lowercase().contains(&*q))
        .cloned()
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "ss-wrapper",

            // Invisible backdrop — closes the panel on outside click.
            if *open.read() {
                div {
                    class: "ss-backdrop",
                    onclick: move |_| {
                        open.set(false);
                        query.set(String::new());
                    },
                }
            }

            // Trigger button.
            button {
                class: "ss-trigger",
                r#type: "button",
                onclick: move |e| {
                    e.stop_propagation();
                    let was_open = *open.peek();
                    open.set(!was_open);
                    if was_open {
                        query.set(String::new());
                    }
                },
                span { class: "ss-trigger-label", "{selected_label}" }
                span { class: "ss-trigger-chevron", "▾" }
            }

            // Drop-down panel.
            if *open.read() {
                div {
                    class: "ss-panel",
                    onclick: move |e| e.stop_propagation(),

                    // Search input.
                    input {
                        class: "ss-search",
                        r#type: "text",
                        placeholder: "Search…",
                        autofocus: true,
                        value: query.read().clone(),
                        oninput: move |e| query.set(e.value()),
                    }

                    // Scrollable option list.
                    div { class: "ss-list",
                        if filtered.is_empty() {
                            div { class: "ss-empty", "No results" }
                        }
                        for option in filtered {
                            {
                                let val = option.value.clone();
                                let label = option.label.clone();
                                let is_selected = value.as_ref().is_some_and(|v| v == &option.value);
                                rsx! {
                                    button {
                                        key: "{label}",
                                        class: if is_selected { "ss-item ss-item--selected" } else { "ss-item" },
                                        r#type: "button",
                                        onclick: move |_| {
                                            on_change.call(val.clone());
                                            open.set(false);
                                            query.set(String::new());
                                        },
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
