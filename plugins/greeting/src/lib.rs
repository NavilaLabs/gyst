//! Zeitrak greeting plugin.
//!
//! Contributes two slot providers:
//!
//! * **`login.greeting`** — full-screen welcome screen shown on the
//!   workspace-select page for 3 seconds after the user logs in.
//! * **`dashboard.widgets`** — a compact "Hello, <email>" card injected into
//!   the dashboard widget row.
//!
//! The host auto-dismisses the `login.greeting` overlay after 3 seconds;
//! the plugin only supplies the visual content.

#![allow(unsafe_code)]

use zeitrak_plugin_sdk::{
    div, h2, p, text, DioxusPlugin, PdkError, PluginCtx, PluginManifest, PluginView, PluginId,
    PriorityHint, SlotProvider, SlotRegistration, plugin,
};

// ── Plugin type ───────────────────────────────────────────────────────────────

/// Root plugin type — carries the manifest and delegates to slot providers.
pub struct GreetingPlugin;

impl DioxusPlugin for GreetingPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest {
            id: PluginId("com.zeitrak.greeting".into()),
            version: "0.1.0".into(),
            slots: vec![
                SlotRegistration {
                    name: "login.greeting".into(),
                    priority_hint: PriorityHint::High,
                },
                SlotRegistration {
                    name: "dashboard.widgets".into(),
                    priority_hint: PriorityHint::Normal,
                },
            ],
            ..Default::default()
        }
    }
}

// ── Login greeting slot ───────────────────────────────────────────────────────

/// Renders the full-screen greeting shown for 3 seconds after login.
///
/// The host wraps this in a dismissible overlay and removes it after the
/// configured timeout; this provider only supplies the visual content.
pub struct LoginGreeting;

impl SlotProvider for LoginGreeting {
    const SLOT_NAME: &'static str = "login.greeting";

    fn render(ctx: &PluginCtx) -> Result<PluginView, PdkError> {
        let display = ctx
            .session
            .email
            .as_deref()
            .unwrap_or("there");

        Ok(
            div()
                .class("greeting-login-screen")
                .child(
                    div()
                        .class("greeting-login-card")
                        .child(
                            h2()
                                .class("greeting-login-title")
                                .child(text(format!("Welcome back, {display}!")))
                                .build(),
                        )
                        .child(
                            p()
                                .class("greeting-login-sub")
                                .child(text("Taking you to your workspaces…"))
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}

// ── Dashboard widget slot ─────────────────────────────────────────────────────

/// Renders a compact greeting card in the dashboard widget row.
pub struct DashboardCard;

impl SlotProvider for DashboardCard {
    const SLOT_NAME: &'static str = "dashboard.widgets";

    fn render(ctx: &PluginCtx) -> Result<PluginView, PdkError> {
        let display = ctx
            .session
            .email
            .as_deref()
            .unwrap_or("there");

        Ok(
            div()
                .class("greeting-dashboard-card")
                .child(
                    p()
                        .class("greeting-dashboard-text")
                        .child(text(format!("Hello, {display}")))
                        .build(),
                )
                .build(),
        )
    }
}

// ── WASM exports ──────────────────────────────────────────────────────────────

plugin! { type: GreetingPlugin, slots: [LoginGreeting, DashboardCard] }
