//! Hook dispatcher — calls plugin pre/post hooks around application-service commands.
//!
//! [`HookDispatcher`] wraps the `HookRegistry` (Phase C, step 12) with the
//! plugin runtime so that application services can call `pre()` and `post()`
//! at their command dispatch points (Phase D, step 17).
//!
//! # Hook key format
//!
//! Hook keys use the form `"<service>.<command>"` with a lower-case service
//! and a lower-case command, e.g. `"timesheet.stop"`.  The dispatcher
//! capitalises the command part when composing the WASM export name, e.g.
//! `hook_timesheet_Stop_Pre`.

use std::sync::{Arc, RwLock};

use dioxus_extism_host::PluginRuntime;
use dioxus_extism_protocol::{HookCall, HookResult, PluginId, SessionCtx};
use serde::{Serialize, de::DeserializeOwned};

use crate::hooks::HookRegistry;
use crate::host_ctx::ZeitrakHostCtx;
use crate::manifest::HookPhase;

// ── Error type ────────────────────────────────────────────────────────────────

/// Error returned when a pre-hook cancels the operation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("hook cancelled by plugin `{plugin_id}`: {reason}")]
pub struct HookCancelled {
    /// The plugin that issued the cancellation.
    pub plugin_id: String,
    /// Human-readable reason from the plugin.
    pub reason: String,
}

// ── Key parsing ───────────────────────────────────────────────────────────────

/// Splits a hook key like `"timesheet.stop"` into `("timesheet", "Stop")`.
///
/// Returns `None` if the key has no `.` separator or the command part is empty.
fn parse_hook_key(key: &str) -> Option<(&str, String)> {
    let (service, raw_command) = key.split_once('.')?;
    if raw_command.is_empty() {
        return None;
    }
    let mut chars = raw_command.chars();
    let command = chars.next().map_or_else(String::new, |first| {
        let upper: String = first.to_uppercase().collect();
        upper + chars.as_str()
    });
    Some((service, command))
}

/// Builds the WASM export name for a hook, e.g. `"hook_timesheet_Stop_Pre"`.
fn hook_fn_name(service: &str, command: &str, phase: &HookPhase) -> String {
    let phase_str = match phase {
        HookPhase::Pre => "Pre",
        HookPhase::Post => "Post",
    };
    format!("hook_{service}_{command}_{phase_str}")
}

// ── HookDispatcher ─────────────────────────────────────────────────────────────

/// Dispatches pre-hooks and post-hooks to plugins at command execution boundaries.
///
/// Application services use this in two patterns (Phase D, §8.4):
///
/// ```rust,ignore
/// // Pre-hook — may cancel or replace the command
/// let cmd = dispatcher.pre("timesheet.stop", cmd, &session, &host_ctx).await?;
///
/// // ... execute command ...
///
/// // Post-hook — fire-and-forget
/// dispatcher.post("timesheet.stop", &result, &session, &host_ctx).await;
/// ```
#[derive(Clone)]
pub struct HookDispatcher {
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
    registry: Arc<RwLock<HookRegistry>>,
}

impl HookDispatcher {
    /// Create a new dispatcher.
    #[must_use]
    pub const fn new(
        runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
        registry: Arc<RwLock<HookRegistry>>,
    ) -> Self {
        Self { runtime, registry }
    }

    /// Run all pre-hooks for `key` in priority order.
    ///
    /// Each pre-hook receives the current command as JSON, and may:
    /// - **Continue** — pass through (optionally mutating the context).
    /// - **Replace** — replace the command with new context.
    /// - **Cancel** — abort the operation; `Err(HookCancelled)` is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err(HookCancelled)` when any pre-hook cancels the operation.
    pub async fn pre<I>(
        &self,
        key: &str,
        cmd: I,
        session: &SessionCtx,
        host_ctx: &ZeitrakHostCtx,
    ) -> Result<I, HookCancelled>
    where
        I: Serialize + DeserializeOwned,
    {
        let Some((service, command)) = parse_hook_key(key) else {
            tracing::warn!(
                key,
                "HookDispatcher::pre: invalid hook key format, skipping"
            );
            return Ok(cmd);
        };

        let hooks: Vec<(PluginId, String)> = {
            let Ok(reg) = self.registry.read() else {
                return Ok(cmd);
            };
            reg.lookup(service, &command, &HookPhase::Pre)
                .into_iter()
                .map(|h| {
                    (
                        PluginId(h.plugin_id.clone()),
                        hook_fn_name(service, &command, &HookPhase::Pre),
                    )
                })
                .collect()
        };

        if hooks.is_empty() {
            return Ok(cmd);
        }

        let mut context = serde_json::to_value(&cmd).unwrap_or_default();

        for (plugin_id, fn_name) in hooks {
            let hook_call = HookCall {
                hook_name: key.to_string(),
                context: context.clone(),
            };

            match self
                .runtime
                .call_plugin::<HookCall, HookResult>(
                    &plugin_id, &fn_name, &hook_call, session, host_ctx,
                )
                .await
            {
                Ok(HookResult::Continue { context: c } | HookResult::Replace { context: c }) => {
                    context = c;
                }
                Ok(HookResult::Cancel { reason }) => {
                    return Err(HookCancelled {
                        plugin_id: plugin_id.0,
                        reason,
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        plugin_id = %plugin_id.0,
                        fn_name,
                        error = %e,
                        "pre-hook call failed — skipping plugin"
                    );
                }
            }
        }

        serde_json::from_value(context).map_err(|e| HookCancelled {
            plugin_id: "<deserialize>".to_string(),
            reason: format!("pre-hook context could not be deserialised back to command: {e}"),
        })
    }

    /// Run all post-hooks for `key`, fire-and-forget.
    ///
    /// Errors from individual hooks are logged but never returned.
    pub async fn post<I>(
        &self,
        key: &str,
        result: &I,
        session: &SessionCtx,
        host_ctx: &ZeitrakHostCtx,
    ) where
        I: Serialize + Sync,
    {
        let Some((service, command)) = parse_hook_key(key) else {
            tracing::warn!(
                key,
                "HookDispatcher::post: invalid hook key format, skipping"
            );
            return;
        };

        let hooks: Vec<(PluginId, String)> = {
            let Ok(reg) = self.registry.read() else {
                return;
            };
            reg.lookup(service, &command, &HookPhase::Post)
                .into_iter()
                .map(|h| {
                    (
                        PluginId(h.plugin_id.clone()),
                        hook_fn_name(service, &command, &HookPhase::Post),
                    )
                })
                .collect()
        };

        let context = serde_json::to_value(result).unwrap_or_default();

        for (plugin_id, fn_name) in hooks {
            let hook_call = HookCall {
                hook_name: key.to_string(),
                context: context.clone(),
            };
            if let Err(e) = self
                .runtime
                .call_plugin::<HookCall, serde_json::Value>(
                    &plugin_id, &fn_name, &hook_call, session, host_ctx,
                )
                .await
            {
                tracing::warn!(
                    plugin_id = %plugin_id.0,
                    fn_name,
                    error = %e,
                    "post-hook call failed"
                );
            }
        }
    }
}

impl std::fmt::Debug for HookDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookDispatcher").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hook_key_capitalises_command() {
        let result = parse_hook_key("timesheet.stop");
        assert_eq!(result, Some(("timesheet", "Stop".to_string())));
    }

    #[test]
    fn parse_hook_key_handles_multi_part_command() {
        let result = parse_hook_key("workspace_role.grantPermission");
        assert_eq!(
            result,
            Some(("workspace_role", "GrantPermission".to_string()))
        );
    }

    #[test]
    fn parse_hook_key_returns_none_for_missing_dot() {
        assert!(parse_hook_key("noservice").is_none());
    }

    #[test]
    fn parse_hook_key_returns_none_for_empty_command() {
        assert!(parse_hook_key("timesheet.").is_none());
    }

    #[test]
    fn hook_fn_name_pre() {
        assert_eq!(
            hook_fn_name("timesheet", "Stop", &HookPhase::Pre),
            "hook_timesheet_Stop_Pre"
        );
    }

    #[test]
    fn hook_fn_name_post() {
        assert_eq!(
            hook_fn_name("activity", "Create", &HookPhase::Post),
            "hook_activity_Create_Post"
        );
    }
}
