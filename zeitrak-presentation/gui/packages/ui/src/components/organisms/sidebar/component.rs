use dioxus::prelude::*;
use dioxus_extism_frontend::PluginSlot;
use dioxus_extism_macros::overridable;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiChevronLeft, HiChevronRight, HiClock, HiCog, HiHashtag, HiHome, HiLogout, HiPlay, HiStop,
    HiTag,
};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

use crate::PluginHostCtx;
use crate::components::atoms::{Button, ButtonVariant, Navbar, NavbarItem, ToastMessage, Toasts};

/// Mirrors the `AuthState` type alias from the `web` crate.
type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

/// Extract up-to-two uppercase initials from an email address.
fn email_initials(email: &str) -> String {
    let local = email.split('@').next().unwrap_or(email);
    let parts: Vec<&str> = local.split(['.', '_', '-']).collect();
    match parts.as_slice() {
        [a, b, ..] => {
            let a = a.chars().next().unwrap_or('?').to_uppercase().to_string();
            let b = b.chars().next().unwrap_or('?').to_uppercase().to_string();
            format!("{a}{b}")
        }
        [a] => a
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string()),
        [] => "?".to_string(),
    }
}

#[overridable]
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

    let initials = email_initials(&user.email);
    let email = user.email.clone();

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

                // ── Navigation grouped by section ──────────────────────────────
                nav { class: "sidebar-nav-groups",
                    // Tracking section
                    div { class: "sidebar-nav-group",
                        span { class: "sidebar-nav-section sidebar-label",
                            {tid!("sidebar-section-tracking")}
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
                        }
                    }

                    // Library section
                    div { class: "sidebar-nav-group",
                        span { class: "sidebar-nav-section sidebar-label",
                            {tid!("sidebar-section-library")}
                        }
                        Navbar {
                            class: "sidebar-nav",
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
                        }
                    }

                    // Preferences section
                    div { class: "sidebar-nav-group",
                        span { class: "sidebar-nav-section sidebar-label",
                            {tid!("sidebar-section-preferences")}
                        }
                        Navbar {
                            class: "sidebar-nav",
                            NavbarItem {
                                index: 4usize,
                                value: "settings".to_string(),
                                to: "/settings",
                                Icon { icon: HiCog, width: 16, height: 16 }
                                span { class: "sidebar-label", {tid!("sidebar-nav-settings")} }
                            }
                        }
                    }

                    // Plugin-contributed sidebar entries (§12.2 — sidebar.entries).
                    PluginSlot::<PluginHostCtx> { name: "sidebar.entries".to_string() }

                    // Plugin-contributed admin menu entries — only visible to admins (§12.2 — admin.menu).
                    if user.is_admin {
                        PluginSlot::<PluginHostCtx> { name: "admin.menu".to_string() }
                    }
                }
            }

            // ── Timer section ──────────────────────────────────────────────────
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

            // ── User strip ────────────────────────────────────────────────────
            div { class: "sidebar-user-strip sidebar-label",
                div { class: "sidebar-user-avatar", "{initials}" }
                div { class: "sidebar-user-meta",
                    span { class: "sidebar-user-email", "{email}" }
                }
            }

            // ── Footer (logout) ───────────────────────────────────────────────
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
