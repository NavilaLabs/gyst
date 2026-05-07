use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::HiMenu;
use dioxus_free_icons::Icon;

use crate::components::molecules::SettingsMenu;

#[component]
pub fn Header(
    /// Current page title shown on the left of the header bar.
    /// Pass an empty string to show only the actions area (e.g. on login/setup).
    #[props(default)]
    title: String,
) -> Element {
    let mut sidebar_open: crate::SidebarOpen = use_context();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        header { class: "header",
            div { class: "header-content",
                // Mobile-only hamburger — hidden on desktop via CSS
                button {
                    class: "header-menu-btn",
                    onclick: move |_| {
                        let current = *sidebar_open.read();
                        sidebar_open.set(!current);
                    },
                    Icon { icon: HiMenu, width: 20, height: 20 }
                }
                if !title.is_empty() {
                    h1 { class: "header-title", "{title}" }
                }
                div { class: "header-actions",
                    SettingsMenu {}
                }
            }
        }
    }
}
