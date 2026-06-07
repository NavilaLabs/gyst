use dioxus::prelude::*;

use crate::layouts::default::WorkspaceAccent;
use crate::NavDirection;

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

    // Read navigation direction at mount time (non-reactive peek so re-renders from other
    // signal changes don't re-trigger the CSS animation). Falls back to forward (0) when
    // used outside the Layout context (e.g. tests, Storybook).
    let dir = try_use_context::<NavDirection>()
        .map(|s| *s.peek())
        .unwrap_or(0i8);
    let slide_class = if dir < 0 {
        "page-enter-backward"
    } else {
        "page-enter-forward"
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div {
            style: accent().as_css_var(),
            class: "default-layout {slide_class}",
            div { class: "default-layout-content", {children} }
        }
    }
}
