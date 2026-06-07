# RFC: Zeitrak Plugin Platform

**Status**: Proposal
**Date**: 2026-06-05
**Scope**: New crate `zeitrak-plugin-host`, additions to `zeitrak-core`, `zeitrak-infrastructure`, `zeitrak-infrastructure-impl`, `zeitrak`, `zeitrak-presentation/gui`
**Depends on**: `dioxus-extism` ≥ release that includes `host-agnostic-extensions`, `host-context-generic`, and `plugin-to-plugin-interaction`

---

## 1. Motivation

Zeitrak today is a closed activity-tracking application. The crate `zeitrak-core/src/plugin.rs` contains only a skeletal `ZeitrakPlugin` trait — there is no actual extension point. We want zeitrak to become a **fully extensible platform**: third parties ship WASM plugins (via Extism, routed through `dioxus-extism`) that extend:

- **Backend**: domain-event reactions, application-service pre/post hooks, plugin-authored event-sourced aggregates, plugin-owned projections, plugin-owned HTTP API routes.
- **Frontend**: slot insertion, named-component replacement, route-wrap / route-inject / route-replace, completely new plugin pages.

**Strict design constraint**: `dioxus-extism` stays a generic plugin runtime with **zero zeitrak vocabulary**. All zeitrak-specific concepts (aggregates, event-sourcing terminology, permission strings, trust tiers) live in a new `zeitrak-plugin-host` crate. The crate uses **only** generic extension mechanisms exposed by `dioxus-extism`:

- `PluginManifest::extensions` + `ManifestExtensionHandler` (host-agnostic manifest extensions),
- `runtime.call_plugin::<I, O>(...)` (generic WASM-function dispatch),
- `HostCapability::Custom { namespace, value }` (host-defined capability classes),
- `TransformOp::RouteReplace` + `RouteReplacePolicyFn<HostCtx>` (route replacement),
- Opaque `TrustTag` (signature verification result),
- `requires_plugins` + `[exports.public]` + `GrantPolicyFn<HostCtx>` (plugin-to-plugin contracts),
- `CallContext<'_, HostCtx>` (host-owned per-call context).

---

## 2. Non-Goals

- We **do not** modify the public API of `dioxus-extism`, `eventually-rs`, `eventually-any`, or `eventually-projection`.
- We **do not** introduce a domain-specific language for plugins. Plugin authors write Rust + Extism PDK.
- We **do not** support live multi-version coexistence: one version per plugin id at runtime (matches `dioxus-extism` constraint).
- Multi-tenant **cross-tenant** plugin reads are forbidden by construction — plugin storage and plugin aggregates are workspace-scoped unless the plugin holds an explicit Admin trust tag.
- We **do not** expose an admin "plugin marketplace UI" in this RFC. Admin install-flow UI is a follow-up.

---

## 3. Architecture overview

### 3.1 Crate boundary

A new crate `zeitrak-plugin-host` sits between `zeitrak-infrastructure-impl` and the application-facade crate `zeitrak`. Onion rule unchanged: `zeitrak-core` stays I/O-free; `zeitrak-plugin-host` may depend on `zeitrak-core`, `zeitrak-infrastructure`, `zeitrak-infrastructure-impl`, `dioxus-extism`, and `eventually-rs` family crates.

```
zeitrak-core
    ↑
zeitrak-infrastructure (port traits)
    ↑
zeitrak-infrastructure-impl (adapters)
    ↑
zeitrak-plugin-host          ← new crate
    ↑
zeitrak (facade, application services)
    ↑
zeitrak-presentation/gui
```

`zeitrak-plugin-host` exposes a single public type `PluginHost` plus the manifest extension handlers, capability mapper, event bus, hook registry, aggregate registry, storage service, and trust-policy mapper.

### 3.2 Three plugin impact tiers

| Tier | Mechanism | Example |
|---|---|---|
| **Reactive** | Subscribe to domain events (read-only, async) | "Notify Slack on `TimesheetStopped`" |
| **Interceptive** | Pre/Post hooks on application-service commands. Return `Continue`/`Cancel`/`Replace`. | "Block `timesheet.stop` if description is empty" |
| **Constructive** | Own event-sourced aggregates + own projections + own API routes + own UI slots/pages | "Custom aggregate `LeaveRequest` with own dashboard" |

A plugin may participate in any combination of tiers.

### 3.3 Three trust tiers (zeitrak policy, not dioxus-extism)

`dioxus-extism` produces an **opaque** `TrustTag { verified: bool, signer_key_id: Option<String> }`. `zeitrak-plugin-host` maps that opaque tag + the install context to a zeitrak-specific trust tier:

| Tier | Installable by | Allowed capabilities (zeitrak policy) |
|---|---|---|
| **Tenant-Plugin** | Workspace admin | Tenant-scope only. No filesystem, no outbound network, no admin-scope reads. |
| **Instance-Plugin** | Instance admin (CLI) | Tenant + admin read-scope. |
| **Signed Instance-Plugin** | Ed25519 signature against configured trust root | Full access including admin writes. Required for route-replace on core routes. |

This mapping lives entirely in `zeitrak-plugin-host::trust`. `dioxus-extism` knows nothing about these tiers.

---

## 4. Prerequisites (must land before plugin-host scaffold)

### 4.1 F1 (P0) — `EventUpcaster` trait

Plugin-authored events will be versioned. Today `event_streams.schema_version` is written but never read on load. Without upcasting, plugin event evolution is impossible.

Add to `zeitrak-core/src/event_upcaster.rs`:

```rust
/// Migrates a persisted event payload from one schema version to the current one.
#[non_exhaustive]
pub trait EventUpcaster: Send + Sync {
    /// The event-type discriminator this upcaster handles (e.g. `"plugin.<id>.LeaveSubmitted"`).
    fn event_type(&self) -> &str;

    /// Highest source version this upcaster understands.
    fn supported_from(&self) -> u32;

    /// Target version after applying this upcaster. Must be strictly greater than `from`.
    fn upcasts_to(&self) -> u32;

    /// Transform a payload. May be called multiple times by a chain.
    fn upcast(&self, from: u32, payload: serde_json::Value) -> Result<serde_json::Value, UpcastError>;
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UpcastError {
    #[error("unsupported source version {from} for event {event_type}")]
    UnsupportedVersion { event_type: String, from: u32 },
    #[error("payload migration failed: {0}")]
    Migration(String),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}
```

Wire dispatch in the `eventually-any`-backed repository load path inside `zeitrak-infrastructure-impl/src/sea_query_sqlx/<scope>/.../repositories.rs`: before deserialising a stored event payload, look up any registered `EventUpcaster` chain for `(event_type, persisted_version)` and apply them in order until the payload version matches the current event's expected version.

### 4.2 F2 — User `rehydrate_from_state(0, user)` bug

In `zeitrak-infrastructure-impl/src/sea_query_sqlx/admin/user/repositories.rs`, replace the hardcoded `0` with the actual snapshot version. Without this fix every user-load replays the entire stream.

### 4.3 F3 — Configurable admin role name

In `zeitrak/src/authorization.rs:52` the literal `"admin"` role name is hardcoded and case-sensitive. Replace by either:

- Env var `ZEITRAK_ADMIN_ROLE_NAME` (default `"admin"`), or
- A dedicated `Permission::ADMIN_BYPASS` permission that the admin role grants.

The latter is preferred — it removes a string-coupled invariant and makes the admin check uniform with other permission checks.

### 4.4 F4 — Invitation integration tests

Add `zeitrak-infrastructure-impl/tests/integration/invitation/` test suite mirroring the layout of `activity/` and `timesheet/`. Cover create / accept / reject / revoke / resend flows.

### 4.5 F5 — Snapshot strategy metadata

Add `snapshot_every: u32` metadata per aggregate as part of `Aggregate` companion data. Used by zeitrak-core's snapshot decision logic and by plugin-authored aggregates (which declare it in their manifest, §8.3).

### 4.6 F6 — Refresh-token decision

Either implement a refresh-token endpoint, or document the 1-hour hard cut and accept it. This is a decision item; do not block plugin work waiting for the implementation if the documented-cut option is chosen.

### 4.7 P2 items

F7–F14 from the Phase-1 review are **not** prerequisites for the plugin platform and may follow after.

---

## 5. New crate: `zeitrak-plugin-host`

### 5.1 Layout

```
zeitrak-plugin-host/
├── Cargo.toml
└── src/
    ├── lib.rs                  // PluginHost facade, re-exports
    ├── lifecycle.rs            // load/reload/unload, calls dioxus-extism PluginRuntime
    ├── manifest.rs             // zeitrak.* manifest extension definitions (typed)
    ├── manifest_handlers.rs    // ManifestExtensionHandler implementations
    ├── trust.rs                // TrustTag → ZeitrakTrustTier mapping
    ├── capabilities.rs         // ZeitrakCapability + CallContext bridge
    ├── host_ctx.rs             // ZeitrakHostCtx (passed as HostCtx to dioxus-extism)
    ├── event_bus.rs            // DomainEvent enum + broadcast bus
    ├── hooks.rs                // Pre/Post hook registry + dispatch
    ├── aggregate_host.rs       // Plugin-owned aggregate registry + Aggregate impl
    ├── projector_bridge.rs     // WASM projection → eventually-projection Projector
    ├── storage.rs              // PluginStorageService (KV + migrations + scoped query)
    ├── audit.rs                // CrossPluginAuditSink impl writing to plugin_audit table
    ├── api.rs                  // axum router for plugin-contributed HTTP routes
    └── error.rs                // PluginHostError
```

### 5.2 `PluginHost` facade

```rust
pub struct PluginHost {
    runtime: Arc<dioxus_extism::PluginRuntime<ZeitrakHostCtx>>,
    event_bus: Arc<EventBus>,
    hooks: Arc<HookRegistry>,
    aggregates: Arc<AggregateRegistry>,
    storage: Arc<PluginStorageService>,
    audit: Arc<PluginAuditSink>,
}
```

`PluginHost::new(...)` constructs the runtime with all `zeitrak.*` extension handlers registered and the capability check / grant policy / route-replace policy hooked into the trust + capability pipeline.

### 5.3 `ZeitrakHostCtx`

This is the `HostCtx` parameter handed to `dioxus-extism::PluginRuntime`. It contains everything the host-side callbacks need to make policy decisions:

```rust
#[derive(Clone)]
pub struct ZeitrakHostCtx {
    pub user_id: Option<UserId>,
    pub workspace_id: Option<WorkspaceId>,
    pub permissions: Arc<PermissionSet>,
    pub trust_tier: ZeitrakTrustTier,            // resolved per plugin-call from the LoadedPlugin
    pub authz: Arc<AuthorizationService>,
    pub clock: Arc<dyn Clock>,
}
```

The host constructs this once per inbound request (HTTP, server-function, projection-run) and threads it into every `runtime.call_plugin(..., &session_ctx, &host_ctx).await` call.

**Construction site**: a single helper `zeitrak_plugin_host::host_ctx::from_session(&Session) -> ZeitrakHostCtx` extracts the values from the existing zeitrak session. Tests construct it directly with explicit fields.

---

## 6. Plugin manifest schema

Plugin manifests have two clearly separated parts:

### 6.1 Part A — `dioxus-extism` core (host-agnostic)

```toml
[plugin]
id = "com.acme.leave-requests"
version = "0.1.0"

[trust]
signature = "..."   # optional Ed25519
signer    = "..."

[[capabilities]]
type = "Custom"
namespace = "zeitrak.permission"
value = "timesheet.read_all"

[[ui_slots]]
slot = "dashboard.widgets"
priority = 100

[[routes]]
path = "/plugin/leave/dashboard"

[exports.public]
get_pending_count = { description = "Returns pending requests for current user" }
```

### 6.2 Part B — `zeitrak.*` extensions

These live under `[extensions."zeitrak.*"]`. From `dioxus-extism`'s perspective they are opaque JSON, parsed by handlers registered by `zeitrak-plugin-host`.

```toml
[extensions."zeitrak.app"]
min_version = "0.5"

[extensions."zeitrak.permissions"]
contributed = ["leave.submit", "leave.approve"]

[extensions."zeitrak.events"]
subscriptions = ["TimesheetStopped", "ActivityCreated"]

[extensions."zeitrak.hooks"]
command_hooks = [
  { service = "timesheet", command = "Stop", phase = "Pre", priority = 100 },
]

[[extensions."zeitrak.aggregates"]]
name = "leave_request"
events = ["Submitted", "Approved", "Rejected"]
snapshot_every = 50
commands = [
  { name = "Submit",  permission = "leave.submit" },
  { name = "Approve", permission = "leave.approve" },
  { name = "Reject",  permission = "leave.approve" },
]

[[extensions."zeitrak.projections"]]
name = "pending_leaves"
table = "pending_leaves"        # becomes plugin_<id>__pending_leaves
events = ["Submitted", "Approved", "Rejected"]
```

### 6.3 Handler responsibilities

For each `zeitrak.*` namespace, `zeitrak-plugin-host::manifest_handlers` registers a `ManifestExtensionHandler` with `dioxus-extism::PluginRuntime`:

| Namespace | Handler's job in `validate` | Handler's job in `on_load` |
|---|---|---|
| `zeitrak.app` | Reject if `min_version` > current zeitrak version | nothing |
| `zeitrak.permissions` | Reject reserved-prefix permission names | Register plugin-contributed permissions in `PermissionRegistry` |
| `zeitrak.events` | Reject unknown event names | Subscribe plugin id to the event bus for those events |
| `zeitrak.hooks` | Reject unknown service/command names | Register hook entries in `HookRegistry` |
| `zeitrak.aggregates` | Reject duplicate aggregate names, validate snapshot policy | Register `(plugin_id, aggregate_name)` in `AggregateRegistry`; verify plugin exports `<name>__apply` and `<name>__handle_command` |
| `zeitrak.projections` | Reject duplicate projection names | Register projection in `projector_bridge`; run plugin migration to create `plugin_<id>__<table>` |

`on_unload` of every handler must reverse its `on_load` work (remove permissions, unsubscribe, deregister, etc.). Projection tables are **not** dropped on unload — only on explicit uninstall + `drop_tables: true`.

---

## 7. Capability bridge

Plugins request zeitrak-specific permissions via `HostCapability::Custom`:

```toml
[[capabilities]]
type = "Custom"
namespace = "zeitrak.permission"
value = "timesheet.read_all"
```

`zeitrak-plugin-host::capabilities` registers a single `CapabilityCheckFn<ZeitrakHostCtx>` with `runtime.register_capability_check_ctx("zeitrak.permission", ...)`:

```rust
runtime.register_capability_check_ctx(
    "zeitrak.permission",
    Arc::new(|plugin_id, value, ctx| {
        let permission = value.as_str()
            .ok_or_else(|| "expected string".to_string())?;

        // 1. Admin-tier check: admin permissions only for Instance / Signed trust tiers.
        if is_admin_permission(permission) {
            match ctx.host.trust_tier {
                ZeitrakTrustTier::Tenant => return Err("tenant plugin may not hold admin permission".into()),
                _ => {}
            }
        }

        // 2. Runtime check against zeitrak's AuthorizationService for the calling user.
        if !ctx.host.permissions.contains(permission) {
            return Err(format!("permission denied: {permission}"));
        }

        Ok(())
    }),
);
```

Plugins that hold no zeitrak-mapped capability for a given action are default-denied — matches `dioxus-extism`'s `HostCapability::Custom` semantics.

### 7.1 Other namespaces

| Namespace | Meaning |
|---|---|
| `zeitrak.permission` | Plugin requests a single zeitrak permission |
| `zeitrak.scope` | One of `"tenant"` (default) or `"admin"`. `admin` requires Instance/Signed tier. |
| `zeitrak.aggregate.write` | Plugin requests write access to its own plugin-aggregate stream. Implicit for plugins that declare `zeitrak.aggregates`. |
| `zeitrak.event.emit` | Plugin requests to publish synthesised domain events (rare). |

---

## 8. Domain event bus & application hooks

### 8.1 `DomainEvent` enum

Define `zeitrak_plugin_host::event_bus::DomainEvent` as a `serde`-serialisable enum that exposes **all** core aggregate events plus a `Plugin { plugin_id, aggregate, event_type, payload }` variant for plugin-authored aggregates. The enum is `#[non_exhaustive]`.

### 8.2 Bus mechanics

- Backed by `tokio::sync::broadcast::Sender<DomainEvent>` with configurable channel capacity (default 1024).
- Events are published **after** the `eventually-any` save commits. Hook into the existing repository save path in `zeitrak-infrastructure-impl/src/sea_query_sqlx/...` via a thin wrapper that:
  1. Calls the inner save.
  2. On success, sends each persisted `Event` to the bus.
- A subscriber drops messages on lag — slow plugins do **not** block the domain operation.
- Each subscription is identified by `(plugin_id, subscription_id)`; a dropped plugin's subscriptions are removed on `on_unload`.

### 8.3 Plugin-side event delivery

Plugins receive events via a new WASM export:

```rust
#[plugin_fn]
pub fn on_domain_event(input: Json<DomainEventEnvelope>) -> FnResult<()> { ... }
```

`DomainEventEnvelope` carries `{ event_name, payload, session_ctx }`. The host dispatches each event via `runtime.call_plugin(plugin_id, "on_domain_event", &envelope, &session, &host_ctx)` for every subscriber matching the event name.

### 8.4 Application-service hooks

Inside `zeitrak/src/{admin,tenant}/*.rs`, every command-handling method gains an explicit hook-dispatch:

```rust
pub async fn stop(&self, cmd: StopTimesheet) -> Result<TimesheetRow> {
    let cmd = self.hooks.pre("timesheet.stop", cmd, &self.host_ctx).await?;
    let result = self.inner_stop(cmd).await;
    self.hooks.post("timesheet.stop", &result, &self.host_ctx).await;
    result
}
```

Pre-hooks return `HookResult::{Continue, Cancel, Replace}` (re-uses `dioxus-extism`'s existing `HookResult`). Post-hooks are fire-and-forget; errors are logged.

Hook ordering is by priority declared in `[extensions."zeitrak.hooks"]`, ties broken by plugin id lexicographic.

Hook dispatch internally calls `runtime.call_plugin(plugin_id, "hook_<service>_<command>_<phase>", &input, &session, &host_ctx)`. The plugin's exported function name must match this convention; the `zeitrak-plugin-sdk` macro hides this from authors (§13).

---

## 9. Plugin-authored event-sourced aggregates

### 9.1 Stream naming

- Core aggregates: `tenant.activity.<uuid>`, `admin.user.<uuid>`.
- Plugin aggregates: `plugin.<plugin_id>.<aggregate_type>.<uuid>`.

The stream prefix is enforced by `zeitrak-plugin-host` when constructing the `eventually-any` `Repository`.

### 9.2 Runtime delegation

For each registered plugin aggregate, `aggregate_host` builds a runtime wrapper that implements `eventually::aggregate::Aggregate` and delegates to WASM exports:

| WASM export (host-side function name) | Signature |
|---|---|
| `<aggregate>__apply` | `Json<(state, event)> -> Json<state>` (pure folder) |
| `<aggregate>__handle_command` | `Json<(state, command)> -> Json<HandleCommandOutput>` (events or error) |
| `<aggregate>__initial_state` | `Json<()> -> Json<state>` |

`HandleCommandOutput` is a small enum: `{ Events(Vec<EventEnvelope>), Error(String) }`. Plugins return `Error` to signal domain rule violations.

### 9.3 Storage

Plugin events are stored in the same event store as core events — `eventually-any` does not distinguish. Snapshots use the existing snapshot repository with `snapshot_every` from the manifest.

### 9.4 Plugin commands from the GUI

A generic HTTP endpoint exposed by `zeitrak`:

```
POST /api/plugin/<plugin_id>/aggregate/<type>/<id>/command
Body: JSON command payload
```

Routing handled by `zeitrak-plugin-host::api`. The endpoint:
1. Resolves the plugin's required permission for `(aggregate, command)` from the manifest.
2. Calls `require_permission`.
3. Loads the aggregate via the wrapper.
4. Calls `handle_command` on the wrapper.
5. Persists the resulting events.
6. Returns 200 / 422 / 403 based on outcome.

### 9.5 Plugin projections

Each declared projection becomes a `Projector` registered with the existing `eventually-projection` runner via `projector_bridge`. The projector calls the plugin's `<projection>__project` export for every matching event. Read tables live under `plugin_<sanitized_id>__<table_name>` and are queryable through `PluginStorageService::query_raw` (§10) with a prefix check.

---

## 10. Plugin storage API

A new service `PluginStorageService` in `zeitrak-plugin-host::storage` offers three modes:

| Mode | Mechanism | Use case |
|---|---|---|
| **State-KV** | Wraps `dioxus-extism`'s `dx_state_*` host functions | UI / session state |
| **Plugin projection tables** | Plugin ships SQL migrations in its bundle; tables are namespaced `plugin_<id>__<name>` | Read models |
| **Plugin aggregate stream** | Automatic via §9 | Write-side state |

API:

```rust
impl PluginStorageService {
    pub async fn kv_get(&self, plugin: &PluginId, scope: StateScope, key: &str)
        -> Result<Option<serde_json::Value>, PluginHostError>;
    pub async fn kv_set(&self, plugin: &PluginId, scope: StateScope, key: &str, value: serde_json::Value)
        -> Result<(), PluginHostError>;
    pub async fn kv_delete(&self, plugin: &PluginId, scope: StateScope, key: &str)
        -> Result<(), PluginHostError>;
    pub async fn migrate(&self, plugin: &PluginId, migration_sql: &str)
        -> Result<(), PluginHostError>;
    pub async fn query_raw(&self, plugin: &PluginId, sql: &str, params: Vec<serde_json::Value>)
        -> Result<Vec<serde_json::Value>, PluginHostError>;
}
```

`query_raw` parses the SQL with a lightweight check: every referenced table must have the prefix `plugin_<sanitized_plugin_id>__`. Any other reference → `PluginHostError::TableAccessDenied`. Use `sqlparser` crate for this — do **not** hand-roll.

`migrate` is executed at plugin install time and on hot-reload. Failed migrations roll back the install.

### 10.1 Pool typing — non-negotiable

`PluginStorageService` is constructed with `Pool<ScopeTenant, StateConnected>` only. There is **no** way to obtain a tenant-only plugin's access to `Pool<ScopeAdmin>`. Instance / Signed tier plugins use a separate `PluginAdminStorageService` that holds a `Pool<ScopeAdmin>` and is only constructed when the loaded plugin's `ZeitrakTrustTier` is Instance or Signed.

This separation is enforced **at type level** — there is no runtime branching on trust tier inside a single storage service.

---

## 11. Trust model

### 11.1 Mapping opaque tag → tier

```rust
fn map_trust_tag(tag: &TrustTag, install_ctx: &InstallContext) -> ZeitrakTrustTier {
    match (tag.verified, install_ctx.installer) {
        (true,  Installer::TrustRoot)     => ZeitrakTrustTier::SignedInstance,
        (false, Installer::InstanceAdmin) => ZeitrakTrustTier::Instance,
        (false, Installer::WorkspaceAdmin) => ZeitrakTrustTier::Tenant,
        _ => ZeitrakTrustTier::Tenant,  // most restrictive default
    }
}
```

`InstallContext::installer` is supplied by whoever calls the install API: the CLI binary supplies `InstanceAdmin`, an admin REST endpoint supplies `WorkspaceAdmin` based on the authenticated session, the boot-loader supplies `TrustRoot` when consuming a signed plugin from the configured trust directory.

### 11.2 Trust-aware policies registered with `dioxus-extism`

- **Route-replace policy**: Tenant plugins may replace only routes under `/plugin/*`. Instance plugins may additionally replace tenant routes (`/timesheet/...`, `/activity/...`). Signed Instance plugins may replace any route.
- **Grant policy** (`GrantPolicyFn<ZeitrakHostCtx>` from `plugin-to-plugin-interaction`): For optional cross-plugin grants, deny when the requesting plugin is Tenant tier and the target is an Instance/Signed plugin (prevents privilege escalation through composition). Required grants are still automatic per the dioxus-extism RFC — if the host wants a hard veto on required-grants it must refuse the install upstream.
- **Capability check policy**: Already covered in §7.

---

## 12. Frontend integration

### 12.1 `HostCtx` provider

In `zeitrak-presentation/gui/packages/ui/src/lib.rs` (top-level layout):

```rust
use_context_provider(|| Arc::new(ZeitrakHostCtx::from_current_session()));
```

`PluginAwareRouter::<ZeitrakHostCtx>`, `PluginSlot::<ZeitrakHostCtx>`, and overridable components consume this via `use_context::<Arc<ZeitrakHostCtx>>()`.

### 12.2 Slot locations (minimum)

Insert `<PluginSlot name="..." />` at:

- `dashboard.widgets`
- `sidebar.entries`
- `activity.detail.tabs`
- `activity.list.toolbar.actions`
- `timesheet.row.actions`
- `timesheet.detail.sections`
- `settings.sections`
- `workspace.settings.sections`
- `admin.menu` (visible only to workspace admins)
- `command-palette.actions` (if/when palette exists)

### 12.3 Component overrides

Mark with `#[overridable("zeitrak.<name>")]`:

- `TimesheetRow`, `ActivityCard`, `DashboardWidget`
- `UserAvatar`, `WorkspaceSwitcher`, `Sidebar`, `TopBar`, `BreadcrumbBar`
- `UserProfileForm`, `WorkspaceSettingsForm`

### 12.4 Plugin routes

Add catch-all `/plugin/:plugin_id/*rest` to the host router. Server function `get_plugin_page` calls `PluginHost::render_page`.

### 12.5 Route-replace

Already supported by `dioxus-extism::TransformOp::RouteReplace`. zeitrak's `RouteReplacePolicyFn` (§11.2) gates which plugins may replace which routes.

---

## 13. Plugin SDK helper crate

Create `zeitrak-plugin-sdk` — **a thin convenience layer for plugin authors**, separate from `zeitrak-plugin-host`. The SDK has no dependency on `zeitrak-core` or any zeitrak server-side code; it depends only on `dioxus-extism-pdk` and re-exports zeitrak-specific schema types (the JSON payload shapes for `zeitrak.aggregates`, `zeitrak.hooks`, `DomainEvent`, etc.) so plugin code stays type-safe.

Macros:

```rust
zeitrak_aggregate! {
    name: leave_request,
    state: LeaveRequestState,
    events: [Submitted, Approved, Rejected],
    snapshot_every: 50,
}

zeitrak_projection! {
    name: pending_leaves,
    table: pending_leaves,
    events: [Submitted, Approved, Rejected],
    project: |state, event| { /* user code */ },
}

zeitrak_hook! {
    service: "timesheet",
    command: "Stop",
    phase: Pre,
    handler: |cmd, ctx| { /* user code returning HookResult */ },
}
```

These generate the correctly-named WASM exports and the corresponding manifest TOML fragments.

---

## 14. Security & sandbox

Layered defence:

1. **WASM sandbox** — Extism / wasmtime: no FS, no network outside host functions.
2. **Capability default-deny** — Already enforced by `dioxus-extism` for declared capabilities.
3. **Trust tier gating** — Per §7 / §11.
4. **Audit log** — `plugin_audit` table in admin DB with columns `(plugin_id, user_id, workspace_id, action, outcome, timestamp, payload_hash)`. Written by:
   - The capability check callback (for denied calls).
   - `PluginAuditSink` registered as `CrossPluginAuditSink` with `dioxus-extism` (per `plugin-to-plugin-interaction.md` §8).
   - Hook dispatcher for `Cancel`/`Replace` outcomes.
5. **Quotas** — Per-plugin call timeout (default 5 s), max pool slots, max memory (Extism `MemoryOptions`). Configurable per trust tier.
6. **Workspace isolation** — Plugin KV state and plugin projection tables are workspace-scoped via the `Pool<ScopeTenant>` typing. Cross-tenant access is impossible at type level.
7. **Plugin uninstall** — Two modes:
   - `disable`: events stay, no new calls, projections paused.
   - `uninstall`: trigger plugin's `on_unload` cleanup hook, optionally drop projection tables (`drop_tables: true`); event streams **are retained** (event sourcing).

---

## 15. Reference plugin

Build `com.acme.leave-requests` as the end-to-end verification:

- **Constructive**: Aggregate `leave_request` with events `Submitted`/`Approved`/`Rejected`. Projection `pending_leaves`. Page route `/plugin/leave/dashboard`.
- **Reactive**: Subscribes to `TimesheetStopped`, decrements a derived vacation-day counter.
- **Interceptive**: Pre-hook on `timesheet.start` — cancels with reason "user is on leave" if user has an approved leave covering now.
- **UI**: Contributes to `dashboard.widgets` ("Pending Leave Requests") and `sidebar.entries` ("Urlaub").
- **Trust**: Tenant-tier (no admin capability) — installable by a workspace admin.

The plugin lives under `examples/plugins/leave-requests/` inside the zeitrak repo. Its build artifact (a `.wasm` plus `plugin.toml`) is loaded by an integration test in `zeitrak-plugin-host/tests/leave_requests_e2e.rs`.

---

## 16. Migration & compatibility

- The existing `zeitrak-core/src/plugin.rs` skeleton (`ZeitrakPlugin` trait, `PluginRegistry`) is **removed**. Nothing in production calls it; its only references are within itself.
- All new public APIs are additive to `zeitrak-core` (`EventUpcaster`) and to the facade (`PluginHost`, server functions in `packages/api/src/plugin/*`).
- `zeitrak-infrastructure-impl` gains an event-bus publishing wrapper around its repository save path. Existing call sites do not change.
- F3 (admin role name) is a **behaviour change**: deployments that rely on the literal `"admin"` role must set the env var or migrate to `Permission::ADMIN_BYPASS`. Document in `CHANGELOG.md` and add an upgrade note.

---

## 17. Open questions

1. **Refresh tokens** (F6) — implement or document the 1-hour cut?
2. **Tenant remote sync** — plugin storage for workspaces that sync to remote PostgreSQL: out of scope for v1, but the `PluginStorageService` design must not preclude it.
3. **Hot-reload of plugin migrations** — Today projection tables are created at install. Schema migrations on plugin upgrade: deferred to a follow-up. v1 requires `uninstall + reinstall` for breaking schema changes.
4. **Plugin-emitted domain events** — Allow plugins to publish synthetic events onto the core bus? Restricted by `zeitrak.event.emit` capability; default-deny. Decision pending: useful for inter-plugin choreography but blurs the "plugin authors only" boundary.
5. **i18n in plugins** — Plugins may want to ship translations. Out of scope for v1; plugins use English-only strings or do their own lookups via host functions.

---

## 18. Implementation plan

Each item is a separate commit / PR. Items in the same phase may proceed in parallel.

### Phase A — Prerequisites
1. F1 `EventUpcaster` trait + dispatch in `eventually-any` load path.
2. F2 user-rehydrate bug fix.
3. F3 admin-role-name resolution (env var or `Permission::ADMIN_BYPASS`).
4. F4 invitation integration tests.
5. F5 `snapshot_every` metadata per core aggregate.
6. F6 refresh-token decision (implement OR document).

### Phase B — `zeitrak-plugin-host` scaffolding
7. Create empty crate. Wire workspace + dependencies. Empty `PluginHost` shell.
8. `ZeitrakHostCtx` + `host_ctx::from_session` helper.
9. `ZeitrakTrustTier` enum + `trust::map_trust_tag`.
10. Capability bridge: `register_capability_check_ctx("zeitrak.permission", ...)`.

### Phase C — Manifest extensions (§6)
11. `zeitrak.app`, `zeitrak.permissions` handlers.
12. `zeitrak.events`, `zeitrak.hooks` handlers.
13. `zeitrak.aggregates`, `zeitrak.projections` handlers (registration only; runtime delegation in Phase E).

### Phase D — Event bus + hooks (§8)
14. `DomainEvent` enum + `EventBus`.
15. Repository save-path wrapper that publishes events after commit.
16. `HookRegistry` + dispatch helpers.
17. Wire hook dispatch into every application-service method in `zeitrak/src/{admin,tenant}/*.rs`.
18. PDK-side: `on_domain_event` export support, `zeitrak_hook!` macro in `zeitrak-plugin-sdk`.

### Phase E — Plugin aggregates (§9)
19. `AggregateRegistry` + WASM-backed `Aggregate` wrapper.
20. `aggregate_host` integration with `eventually-any` Repository (stream-prefix).
21. `projector_bridge` integration with `eventually-projection` runner.
22. HTTP endpoint `POST /api/plugin/.../command`.
23. `zeitrak_aggregate!` + `zeitrak_projection!` macros in `zeitrak-plugin-sdk`.

### Phase F — Storage (§10)
24. `PluginStorageService` + `PluginAdminStorageService` (separate types per pool scope).
25. Plugin migration runner at install time.
26. `query_raw` with prefix-checked table whitelist (`sqlparser`-based).

### Phase G — Frontend (§12)
27. `ZeitrakHostCtx` provider in root layout.
28. Insert all `<PluginSlot>` locations.
29. Annotate strategic components with `#[overridable]`.
30. Catch-all `/plugin/:plugin_id/*rest` route + `get_plugin_page` server function.
31. Route-replace policy implementation (§11.2).

### Phase H — Security (§14)
32. `plugin_audit` table + migration.
33. `PluginAuditSink` implementation.
34. Per-tier quota defaults wired into `PluginInstallConfig`.

### Phase I — Reference plugin & E2E
35. `examples/plugins/leave-requests/` implementation.
36. `zeitrak-plugin-host/tests/leave_requests_e2e.rs` integration test.
37. Manual GUI walkthrough (Submit → Approve → Pre-hook blocks timesheet → Slot shows status).

---

## 19. Summary

The zeitrak plugin platform is built **on top of** `dioxus-extism`'s host-agnostic primitives, with all zeitrak-specific semantics — event-sourced aggregates, domain event bus, command hooks, permissions, trust tiers, projection storage — confined to a new `zeitrak-plugin-host` crate. `dioxus-extism` learns no zeitrak vocabulary. Three plugin tiers (Reactive, Interceptive, Constructive) plus three trust tiers (Tenant, Instance, Signed Instance) cover the realistic install scenarios. The reference `leave-requests` plugin exercises every tier and serves as the acceptance gate.
