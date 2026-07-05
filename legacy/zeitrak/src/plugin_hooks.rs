//! Global hook dispatcher for application-service pre/post hooks.
//!
//! Application services call [`run_pre`] and [`run_post`] at command
//! boundaries so that loaded plugins can observe (post) or cancel (pre)
//! core operations without the services needing to know about the plugin system.
//!
//! # Lifecycle
//!
//! The hook dispatcher is `None` until the application explicitly calls
//! [`init_hook_dispatcher`] at startup.  When `None`, all hook calls are
//! no-ops; the existing code paths are unaffected.
//!
//! # Pre-hook context mutation
//!
//! Because the current application services are free functions that accept
//! individual parameters (not command structs), pre-hook context mutations
//! are **not propagated back** to the underlying parameters.  Plugins can
//! cancel an operation (returning `Err`) but cannot modify fields.  A future
//! refactor to command structs will enable full pre-hook mutation.

use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use dioxus_extism_protocol::SessionCtx;
use zeitrak_core::shared::clock::SystemClock;
use zeitrak_infrastructure::authorization::{AuthorizationError, AuthorizationRepository};
use zeitrak_plugin_host::hook_dispatcher::{HookCancelled, HookDispatcher};
use zeitrak_plugin_host::host_ctx::{PermissionSet, ZeitrakHostCtx};
use zeitrak_plugin_host::trust::ZeitrakTrustTier;

// ── Global hook dispatcher ────────────────────────────────────────────────────

static HOOK_DISPATCHER: OnceLock<Arc<HookDispatcher>> = OnceLock::new();

/// Initialise the global hook dispatcher.
///
/// Call once at application startup after constructing [`zeitrak_plugin_host::PluginHost`].
/// Subsequent calls are silently ignored (uses [`OnceLock`]).
pub fn init_hook_dispatcher(dispatcher: Arc<HookDispatcher>) {
    let _ = HOOK_DISPATCHER.set(dispatcher);
}

fn hook_dispatcher() -> Option<&'static HookDispatcher> {
    HOOK_DISPATCHER.get().map(Arc::as_ref)
}

// ── System-level hook context ─────────────────────────────────────────────────

/// No-op authorisation backend used for system-level hook calls from free
/// functions that have no authenticated user context.
struct NoOpAuthorizationRepository;

#[async_trait]
impl AuthorizationRepository for NoOpAuthorizationRepository {
    async fn is_admin(&self, _user_id: &str) -> Result<bool, AuthorizationError> {
        Ok(false)
    }

    async fn has_permission(
        &self,
        _user_id: &str,
        _workspace_id: &str,
        _permission: &str,
    ) -> Result<bool, AuthorizationError> {
        Ok(false)
    }

    async fn user_permissions(
        &self,
        _user_id: &str,
        _workspace_id: &str,
    ) -> Result<HashSet<String>, AuthorizationError> {
        Ok(HashSet::new())
    }
}

fn system_host_ctx() -> ZeitrakHostCtx {
    ZeitrakHostCtx {
        user_id: None,
        workspace_id: None,
        permissions: Arc::new(PermissionSet::default()),
        trust_tier: ZeitrakTrustTier::Tenant,
        authz: Arc::new(NoOpAuthorizationRepository),
        clock: Arc::new(SystemClock),
    }
}

// ── Public helpers ────────────────────────────────────────────────────────────

/// Run all Pre-phase hooks for `key` with `ctx` as the opaque command context.
///
/// Returns `Err` if any loaded plugin cancels the operation; `Ok(())` otherwise.
/// If no hook dispatcher has been initialised, this is a no-op.
///
/// # Errors
///
/// Returns [`anyhow::Error`] when a plugin cancels the operation.
pub async fn run_pre(key: &str, ctx: serde_json::Value) -> anyhow::Result<()> {
    let Some(hooks) = hook_dispatcher() else {
        return Ok(());
    };
    hooks
        .pre(key, ctx, &SessionCtx::default(), &system_host_ctx())
        .await
        .map(|_| ())
        .map_err(|e: HookCancelled| anyhow::anyhow!("{e}"))
}

/// Run all Post-phase hooks for `key` fire-and-forget.
///
/// If no hook dispatcher has been initialised, this is a no-op.
pub async fn run_post<T: serde::Serialize + Sync>(key: &str, result: &T) {
    let Some(hooks) = hook_dispatcher() else {
        return;
    };
    hooks
        .post(key, result, &SessionCtx::default(), &system_host_ctx())
        .await;
}
