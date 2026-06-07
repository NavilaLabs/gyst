//! Application-service façade for plugin aggregate commands.
//!
//! Provides [`execute_plugin_command`], the single entry-point for
//! `POST /api/plugin/<plugin_id>/aggregate/<type>/<id>/command` requests.
//!
//! The function:
//! 1. Extracts the shared runtime from the provided [`PluginHost`].
//! 2. Constructs a [`PluginAggregateHost`] for the given plugin and aggregate type,
//!    running any pending event-store migrations.
//! 3. Delegates command execution to `PluginAggregateHost::execute_command`.
//! 4. Returns the list of events that were persisted.
//!
//! Callers are responsible for resolving authentication (JWT → `ZeitrakHostCtx`)
//! and for extracting request parameters from the HTTP layer before calling
//! this function.
//!
//! ## Pool typing
//!
//! Plugin aggregates are always tenant-scoped; pass a [`ConnectedTenantPool`]
//! which is converted to `AnyPool` for the `eventually-any` repository.

use std::sync::Arc;

use anyhow::Context;
use dioxus_extism_protocol::SessionCtx;
use zeitrak_infrastructure_impl::ConnectedTenantPool;
use zeitrak_plugin_host::PluginHost;
use zeitrak_plugin_host::aggregate_host::{PluginAggregateHost, PluginEvent};
use zeitrak_plugin_host::host_ctx::ZeitrakHostCtx;

/// Execute a command on a plugin-authored aggregate.
///
/// # Arguments
///
/// - `plugin_id` — the plugin that owns the aggregate type (e.g. `"my-org/leave-guard"`).
/// - `aggregate_type` — the aggregate type name from the manifest (e.g. `"leave_request"`).
/// - `aggregate_uuid` — the UUID portion of the stream ID (caller-supplied or `uuid::Uuid::new_v4()`).
/// - `command` — JSON-encoded command payload forwarded to `<type>__handle_command`.
/// - `snapshot_every` — taken from `AggregateDecl::snapshot_every`; use `50` as default.
/// - `pool` — tenant pool for the event store.
/// - `plugin_host` — zeitrak plugin host (provides the shared runtime).
/// - `session` — caller session forwarded to WASM.
/// - `host_ctx` — host context forwarded to WASM (used for capability checks).
///
/// # Errors
///
/// Returns [`anyhow::Error`] wrapping a
/// [`PluginCommandError`][zeitrak_plugin_host::aggregate_host::PluginCommandError]
/// on domain/store failures, or a migration error on first use.
#[allow(clippy::too_many_arguments)]
pub async fn execute_plugin_command(
    plugin_id: &str,
    aggregate_type: &str,
    aggregate_uuid: &str,
    command: serde_json::Value,
    snapshot_every: usize,
    pool: &ConnectedTenantPool,
    plugin_host: &PluginHost,
    session: &SessionCtx,
    host_ctx: &ZeitrakHostCtx,
) -> anyhow::Result<Vec<PluginEvent>> {
    let runtime = Arc::clone(plugin_host.runtime());
    let any_pool = pool.as_ref().clone();

    let host =
        PluginAggregateHost::new(any_pool, runtime, plugin_id, aggregate_type, snapshot_every)
            .await
            .context("initialising plugin aggregate repository")?;

    host.execute_command(aggregate_uuid, command, session, host_ctx)
        .await
        .context("executing plugin aggregate command")
}
