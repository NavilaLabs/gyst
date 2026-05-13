use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiChevronLeft, HiChevronRight, HiClock, HiCog, HiHashtag, HiHome, HiLogout, HiPlay, HiStop,
    HiTag,
};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

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
    let mut sidebar_open: crate::SidebarOpen = use_context();

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
    let open = *sidebar_open.read();

    let sidebar_class = if open {
        "sidebar sidebar--open"
    } else {
        "sidebar sidebar--collapsed"
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        // Mobile backdrop — only visible on small screens via CSS
        if open {
            div {
                class: "sidebar-backdrop",
                onclick: move |_| sidebar_open.set(false),
            }
        }

        aside { class: sidebar_class,
            div { class: "sidebar-top",
                div { class: "sidebar-brand",
                    div { class: "sidebar-brand-text",
                        span { class: "sidebar-brand-name", {tid!("sidebar-brand-name")} }
                        span { class: "sidebar-brand-sub", {tid!("sidebar-brand-sub")} }
                    }
                    button {
                        class: "sidebar-collapse-btn",
                        onclick: move |_| sidebar_open.set(!open),
                        if open {
                            Icon { icon: HiChevronLeft, width: 16, height: 16 }
                        } else {
                            Icon { icon: HiChevronRight, width: 16, height: 16 }
                        }
                    }
                }
                Navbar {
                    class: "sidebar-nav",
                    NavbarItem {
                        index: 0usize,
                        value: "dashboard".to_string(),
                        to: "/dashboard",
                        Icon { icon: HiHome, width: 16, height: 16 }
                        span { class: "sidebar-label", {tid!("sidebar-nav-dashboard")} }
                    }
                    NavbarItem {
                        index: 1usize,
                        value: "timesheets".to_string(),
                        to: "/timesheets",
                        Icon { icon: HiClock, width: 16, height: 16 }
                        span { class: "sidebar-label", {tid!("sidebar-nav-timesheets")} }
                    }
                    NavbarItem {
                        index: 2usize,
                        value: "activities".to_string(),
                        to: "/activities",
                        Icon { icon: HiTag, width: 16, height: 16 }
                        span { class: "sidebar-label", {tid!("sidebar-nav-activities")} }
                    }
                    NavbarItem {
                        index: 3usize,
                        value: "tags".to_string(),
                        to: "/tags",
                        Icon { icon: HiHashtag, width: 16, height: 16 }
                        span { class: "sidebar-label", {tid!("sidebar-nav-tags")} }
                    }
                    NavbarItem {
                        index: 4usize,
                        value: "settings".to_string(),
                        to: "/settings",
                        Icon { icon: HiCog, width: 16, height: 16 }
                        span { class: "sidebar-label", {tid!("sidebar-nav-settings")} }
                    }
                }
            }
            div { class: "sidebar-timer",
                if is_running {
                    div { class: "sidebar-timer-running",
                        div { class: "sidebar-timer-info",
                            div { class: "sidebar-timer-indicator",
                                span { class: "sidebar-timer-dot" }
                                span { class: "sidebar-timer-label sidebar-label", {tid!("sidebar-timer-running")} }
                            }
                            span { class: "sidebar-timer-elapsed sidebar-label",
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
                            span { class: "sidebar-label", {tid!("common-stop")} }
                        }
                    }
                } else {
                    Button {
                        onclick: move |_| async move {
                            match api::timesheet::start_timesheet(None, None).await {
                                Ok(dto) => running.set(Some(dto)),
                                Err(e) => toasts.write().push(ToastMessage::error(e.to_string())),
                            }
                        },
                        Icon { icon: HiPlay, width: 14, height: 14 }
                        span { class: "sidebar-label", {tid!("sidebar-start-timer")} }
                    }
                }
            }
            div { class: "sidebar-footer",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: on_logout,
                    Icon { icon: HiLogout, width: 16, height: 16 }
                    span { class: "sidebar-label", {tid!("sidebar-logout")} }
                }
            }
        }
    }
}
