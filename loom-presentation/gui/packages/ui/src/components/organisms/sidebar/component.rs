use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiClock, HiCog, HiHashtag, HiHome, HiLogout, HiPlay, HiStop, HiTag,
};
use dioxus_free_icons::Icon;

use crate::components::atoms::{Button, ButtonVariant, Navbar, NavbarItem, ToastMessage, Toasts};

/// Mirrors the `AuthState` type alias from the `web` crate.
type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn Sidebar() -> Element {
    let auth: AuthState = use_context();
    let user = auth.cloned().flatten();

    let nav = use_navigator();
    let mut auth: AuthState = use_context();
    let mut running: crate::RunningTimer = use_context();
    let mut toasts: Toasts = use_context();

    let on_logout = move |_| async move {
        let _ = api::auth::logout().await;
        auth.set(Some(None));
        nav.replace("/login");
    };

    let elapsed_secs: crate::RunningElapsed = use_context();

    let on_stop = move |_| async move {
        let ts_id = running.peek().as_ref().map(|ts| ts.id.clone());
        if let Some(id) = ts_id {
            match api::timesheet::stop_timesheet(id).await {
                Ok(()) => running.set(None),
                Err(e) => toasts.write().push(ToastMessage::error(e.to_string())),
            }
        }
    };

    // Only render the sidebar when the user has an active workspace session.
    let Some(user) = user else { return rsx! {} };
    if user.workspace_id.is_none() {
        return rsx! {};
    }

    let is_running = running.read().is_some();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        aside { class: "sidebar",
            div { class: "sidebar-top",
                div { class: "sidebar-brand",
                    span { class: "sidebar-brand-name", "Loom" }
                    span { class: "sidebar-brand-sub", "Curated Workspace" }
                }
                Navbar {
                    class: "sidebar-nav",
                    NavbarItem {
                        index: 0usize,
                        value: "dashboard".to_string(),
                        to: "/dashboard",
                        Icon { icon: HiHome, width: 16, height: 16 }
                        "Dashboard"
                    }
                    NavbarItem {
                        index: 1usize,
                        value: "timesheets".to_string(),
                        to: "/timesheets",
                        Icon { icon: HiClock, width: 16, height: 16 }
                        "Timesheets"
                    }
                    NavbarItem {
                        index: 2usize,
                        value: "activities".to_string(),
                        to: "/activities",
                        Icon { icon: HiTag, width: 16, height: 16 }
                        "Activities"
                    }
                    NavbarItem {
                        index: 3usize,
                        value: "tags".to_string(),
                        to: "/tags",
                        Icon { icon: HiHashtag, width: 16, height: 16 }
                        "Tags"
                    }
                    NavbarItem {
                        index: 4usize,
                        value: "settings".to_string(),
                        to: "/settings",
                        Icon { icon: HiCog, width: 16, height: 16 }
                        "Settings"
                    }
                }
            }
            div { class: "sidebar-timer",
                if is_running {
                    div { class: "sidebar-timer-running",
                        div { class: "sidebar-timer-info",
                            div { class: "sidebar-timer-indicator",
                                span { class: "sidebar-timer-dot" }
                                span { class: "sidebar-timer-label", "Timer Running" }
                            }
                            span { class: "sidebar-timer-elapsed",
                                {
                                    let e = *elapsed_secs.read();
                                    format!("{:02}:{:02}:{:02}", e / 3600, (e % 3600) / 60, e % 60)
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: on_stop,
                            Icon { icon: HiStop, width: 14, height: 14 }
                            "Stop"
                        }
                    }
                } else {
                    Button {
                        onclick: move |_| { nav.push("/timesheets"); },
                        Icon { icon: HiPlay, width: 14, height: 14 }
                        "Start Timer"
                    }
                }
            }
            div { class: "sidebar-footer",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: on_logout,
                    Icon { icon: HiLogout, width: 16, height: 16 }
                    "Logout"
                }
            }
        }
    }
}
