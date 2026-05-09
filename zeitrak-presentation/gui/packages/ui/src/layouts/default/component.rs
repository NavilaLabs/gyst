use dioxus::prelude::*;

use crate::layouts::default::WorkspaceAccent;

#[component]
pub fn DefaultLayout(
    /// Optional workspace accent color, e.g. `"#6366f1"`.
    /// Defaults to the brand green defined in `theme.css`.
    #[props(default)]
    accent: Option<String>,
    children: Element,
) -> Element {
    let accent = use_memo(move || accent.clone().map(WorkspaceAccent).unwrap_or_default());
    #[allow(clippy::redundant_closure)]
    use_context_provider(move || accent());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("/src/layouts/default/style.css") }

        div {
            style: accent().as_css_var(),
            class: "default-layout",
            div { class: "default-layout-content", {children} }
        }
    }
}
