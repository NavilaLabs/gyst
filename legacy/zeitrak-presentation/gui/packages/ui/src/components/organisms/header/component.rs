use dioxus::prelude::*;
// use dioxus_extism_macros::overridable;
use dioxus_free_icons::icons::hi_solid_icons::{HiMenu, HiOfficeBuilding};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

use crate::components::molecules::SettingsMenu;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

// #[overridable]
#[component]
pub fn Header(
    /// Current page title shown on the left of the header bar.
    /// Pass an empty string to show only the actions area (e.g. on login/setup).
    #[props(default)]
    title: String,
) -> Element {
    let mut sidebar_open: crate::SidebarOpen = use_context();
    let auth: AuthState = use_context();
    let navigator = use_navigator();

    let has_workspace = auth
        .read()
        .as_ref()
        .and_then(|o| o.as_ref())
        .and_then(|u| u.workspace_id.as_ref())
        .is_some();

    rsx! {
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
                    if has_workspace {
                        button {
                            class: "header-switch-ws-btn",
                            title: tid!("header-switch-workspace"),
                            onclick: move |_| {
                                navigator.push("/select-workspace");
                            },
                            Icon { icon: HiOfficeBuilding, width: 16, height: 16 }
                            span { class: "header-switch-ws-label", {tid!("header-switch-workspace")} }
                        }
                    }
                    SettingsMenu {}
                }
            }
        }
    }
}
