# Zeitrak Architecture Review — May 2026

**Status**: Prerequisite (Phase 1) for the plugin platform initiative.
**Date**: 2026-05-27
**Reviewer**: Claude (automated audit agents, curated)
**Scope**: Verification of architecture conformance, aggregate completeness, auth, event-store integrity, and technical debt — as a baseline before plugin hooks, the domain-event bus, and plugin-authored aggregate hosting are introduced.

---

## Executive Summary

zeitrak is in markedly better shape than the original plan assumed:

- **Architecture**: Onion / Ports & Adapters is strictly enforced; `zeitrak-core` is I/O-free.
- **Aggregate completeness**: 8 of 8 aggregates are effectively CRUD-complete with projectors and API endpoints. `Permission` is intentionally read-only / seed-driven. Only **Invitation** lacks a dedicated integration-test module.
- **Auth & Authz**: JWT validation is hardened (HS256 whitelist, alg:none tests, algorithm-confusion tests). RBAC runs consistently through `AuthorizationService::require_permission`.
- **Multi-tenancy**: Phantom-type-based `Pool<Scope, State>` makes scope crossing impossible at compile time. Tenant DB is bound to a workspace via `workspace_id` embedded in the pool.
- **Event store**: Persistence is cleanly idempotent, snapshots exist, the ProjectionRunner is crash-resilient via `SqlCheckpoint`. **Main gap**: no event upcasting / versioning.

**Consequence for Phase 2 of the plugin plan**: Phase 2 (CRUD completion) is **dramatically smaller than expected**. Only narrow detail gaps (see P1/P2 below) plus the missing event-upcasting mechanism are required prerequisites.

---

## 1. Architecture Conformance (Onion / Ports & Adapters)

**Status: compliant.** No violations found.

| Crate | Dependencies | Finding |
|---|---|---|
| `zeitrak-core` | `async-trait`, `chrono`, `serde`, `uuid`, `validator`, `eventually` | I/O-free. No sqlx/axum/reqwest/tokio::fs. |
| `zeitrak-infrastructure` | `config`, `async-trait`, `chrono`, `tokio`, `url`, `figment` | Pure port-trait definitions. |
| `zeitrak-infrastructure-impl` | `sqlx`, `sea-query`, `sea-query-sqlx`, `jsonwebtoken`, `bcrypt`, `reqwest` | Adapter layer. Correctly depends on both `zeitrak-core` and `zeitrak-infrastructure`. |
| `zeitrak` | composition of all layers | Facade, orchestrates services. |

**Verification**: `grep` for `sqlx`/`axum` inside `zeitrak-core/src/` is empty. Dependency inversion (repository traits in core, implementations in impl) is consistently applied throughout.

---

## 2. Aggregate Completeness

Audited: Activity, Timesheet, TimesheetTag (tenant) and User, Workspace, WorkspaceRole, Permission, Invitation (admin).

| Aggregate | Scope | Commands | Queries | Events | Projector | API | Permissions | Tests |
|---|---|---|---|---|---|---|---|---|
| **Activity** | Tenant | Create, Update, Delete | find_by_id, find_all, list_all | Created, Updated, Deleted | ✓ | ✓ | ACTIVITY_CREATE/UPDATE/DELETE | ✓ (213 LOC) |
| **Timesheet** | Tenant | Start, Stop, Update, Reassign, Cancel, UpdateTime, CreateManual | find_by_id, find_all, filter_* | Started, Stopped, Updated, Reassigned, TimeUpdated, Cancelled | ✓ | ✓ | TIMESHEET_CREATE/UPDATE/CANCEL/EXPORT | ✓ (257 LOC) |
| **TimesheetTag** | Tenant | Create, Rename, Delete, Tag, Untag | find_by_id, find_all, list_by_timesheet | Created, Renamed, Deleted, TimesheetTagged, TimesheetUntagged | ✓ | ✓ | TAG_MANAGE | ✓ (258 LOC) |
| **User** | Admin | Create, UpdateSettings, RequestVerification, VerifyEmail | find_by_id, find_by_email, find_all, list_all | Created, SettingsUpdated, VerificationRequested, Verified | ✓ | ✓ | (auth-specific) | ✓ (350 LOC) |
| **Workspace** | Admin | Create, AssignUserRole, RevokeUserRole, GrantUserPermission, RevokeUserPermission, UpdateSettings, RemoveMember | find_by_id, find_all, list_user_workspaces | Created, UserRoleAssigned/Revoked, UserPermissionGranted/Revoked, SettingsUpdated, UserRemoved | ✓ | ✓ | (implicit via role) | ✓ (167 LOC) |
| **WorkspaceRole** | Admin | Create, GrantPermission, RevokePermission, Rename, Delete | find_by_id, find_all, list_by_workspace | Created, PermissionGranted/Revoked, Renamed, Deleted | ✓ | ✓ | ROLE_MANAGE | ✓ (281 LOC) |
| **Permission** | Admin | Create | find_by_id, find_all | Created | ✓ | ✓ (read-only) | — | ✓ (184 LOC) |
| **Invitation** | Admin | Create, Accept, Revoke | find_by_id, find_by_token, list_by_workspace, list_by_email | Created, Accepted, Revoked | ✓ | ✓ (send, list, accept, decline, revoke, register-and-accept) | MEMBER_INVITE | **missing** |

### Gap List

The original plan assumption (User only Settings, Permission only Create, TimesheetTag a skeleton, Workspace without commands) is **outdated** — the aggregates are already built out.

**Actual remaining gaps**:
- **Invitation**: no dedicated integration tests under `zeitrak-infrastructure-impl/tests/integration/`. Functionality is only covered indirectly through other suites.
- **User**: no `SoftDelete`/`Restore`. No `ChangePassword` flow separate from settings (see Auth section).
- **Workspace**: no `Delete` command. Currently no way to delete a workspace.
- **Activity**: no `Restore` (after `Delete`), no `BulkUpdate`.
- **Timesheet**: no hard `Delete` (only `Cancel`).
- **Listing endpoints**: no unified `FilterExpr`/`Page`/`Sort` DSL — currently aggregate-specific `filter_*` functions.

---

## 3. Authentication & Authorization

### JWT Validation
Status: **hardened**.
- Algorithm hardcoded to `HS256` (`zeitrak/src/authentication.rs:56`).
- `alg:none` and algorithm confusion are explicitly rejected by tests (`zeitrak/tests/security/auth_tests.rs:179`, `:163`).
- Token lifetime: 1 hour, with compile-time assert guarding against accidental increase (`zeitrak-infrastructure-impl/src/sea_query_sqlx/admin/authentication.rs:30`).
- **No refresh-token flow** for user sessions. SMTP OAuth2 uses encrypted refresh tokens (`zeitrak-infrastructure-impl/src/smtp/repository.rs`), but end users must log in again after 1 hour.
- **`kid` header is not validated** — only relevant once key rotation / multi-key is needed.

### Authorization
- `RoleBasedPolicy` (`zeitrak/src/authorization.rs:45–127`) checks:
  1. Admin bypass: workspace role with the exact name `"admin"` (case-sensitive, hardcoded at line 52).
  2. Permission grant via role mapping OR direct user grant.
- **Hardcoded admin role name** is the main finding — should become configurable (env var) or be replaced by a dedicated `Permission::ADMIN_BYPASS`.
- **Protected endpoints**: all mutations (Activity Create/Update/Delete, Timesheet, Member, WorkspaceRole, Tag, Invitation).
- **Unprotected read endpoints**: `list_members`, `list_permissions`, `list_workspace_roles`, `list_activities`, `list_timesheets`, `get_invitation_by_token`. Rationale: workspace membership suffices for reads. Defensible, but the privacy model should be documented.
- **SQL Injection**: parameterised queries throughout, verified by tests (`zeitrak/tests/security/authorization_tests.rs:207+`).

---

## 4. Multi-Tenancy Isolation

Status: **strong, compile-time-safe.**

- `Pool<Scope, State>` (`zeitrak-infrastructure-impl/src/sea_query_sqlx/infrastructure/pool.rs:60–114`) carries scope as a phantom type:
  ```rust
  pub struct Pool<Scope, State = StateDisconnected> {
      state: State,
      database_type: DatabaseType,
      scope: PhantomData<Scope>,
      tenant_id: Option<Uuid>,
  }
  ```
- Scopes: `ScopeDefault` (bootstrap), `ScopeAdmin`, `ScopeTenant`. Cross-scope calls are statically impossible.
- The tenant pool is bound to a workspace via `connect_tenant(workspace_id)` (`.../connect.rs:60–67`).
- Workspace ID comes from `session_workspace()` in API handlers and is passed through explicitly.

**Implication for plugins**: plugin-authored aggregates automatically inherit tenant isolation when they reach the tenant pool. The plugin storage API must respect this pool type and must never switch to `Pool<ScopeAdmin>` without an explicit trust check.

---

## 5. Event-Store Integrity

### Persistence & Idempotency
- Schema (`zeitrak-migrations/zeitrak-shared-migrations/src/lib.rs`):
  - `events` table with PK `(event_stream_id, version)` → idempotent.
  - `event_streams` table with PK `event_stream_id` → no duplicate streams.
- Snapshots exist via `SnapshotRepository<A, P>` (`zeitrak-infrastructure-impl/src/snapshot.rs`) with composite index `(aggregate_type, aggregate_id, version)`.
- Projectors use `ON CONFLICT do_nothing()` (e.g. Activity projector line 53) → idempotent replays.

### Snapshot Strategy
- **Not configured**: neither `snapshot_every` per aggregate nor a central policy. Snapshot triggers are currently opaque inside `eventually-any`. For plugin-authored aggregates (plan Phase 5), `snapshot_every` comes from the manifest — the same pattern should be retro-fitted to core aggregates.

### Event Versioning / Upcasting
- The `schema_version` column exists in the DDL but **is never read or validated**. Schema changes would result in silent deserialisation failure.
- **Recommendation**: introduce an `EventUpcaster` trait before launching the plugin platform; otherwise plugin events have no future-proof migration path.

### ProjectionRunner
- One daemon per scope: `zeitrak/src/bin/tenant_projection_daemon.rs`, `admin_projection_daemon.rs`.
- The tenant daemon dispatches sequentially through `TenantProjector` (`zeitrak-infrastructure-impl/src/sea_query_sqlx/tenant/projectors.rs`) — FK-safe ordering.
- Checkpoints via `SqlCheckpoint::new(pool, &name)` with names like `"tenant_projection_{workspace_id}"`. Crash-resilient.
- Unknown event types are **silently ignored** in projector dispatch. With the plugin platform this must remain robust (no crash on unknown plugin events in the core projector).

### Replay Performance
- ✓ Safe via checkpoints — no full replay on restart.
- ⚠ `Root::rehydrate_from_state(0, user) // TODO` bug in `zeitrak-infrastructure-impl/src/sea_query_sqlx/admin/user/repositories.rs:1` forces full replay for user reads. **Must be fixed** (P1).

---

## 6. Test-Coverage Overview

- 17 integration tests across two crates:
  - `zeitrak-infrastructure-impl/tests/integration/`: 12 (user, workspace, activity, timesheet, timesheet_tag, permission, workspace_role, smtp, database).
  - `zeitrak/tests/`: 5 (registration, security/auth, security/authorization).
- No `#[ignore]` tests.
- Six `unimplemented!("test stub")` calls in `ReadRepository` test doubles — intentional, not production code.

### Gaps
- Invitation has **no** dedicated integration test.
- No snapshot recovery tests (verifying snapshots avoid full replay).
- No projection crash-recovery tests (`SqlCheckpoint` resume).
- No multi-workspace concurrency tests.

---

## 7. Technical Debt & WIP

### Branches
- Local: only `main` and the current feature branch `claude/admiring-goldberg-mijsy`.
- **Remote** (verified via GitHub MCP): `desktop`, `main`, `multi-user`, `old`, `refactor`, `refactoring` are present. The local clone does not track these, but `multi-user` and `desktop` matter for the plan (originally assumed). Their state should be checked before merging plugin work — they may already overlap with Invitation/i18n work.

### TODOs
- 1 production TODO: User-aggregate version bypass (see Event-Store section).

### i18n
- The GUI uses `dioxus-i18n` with `tid!()` macros. The backend has no i18n — API error messages are hardcoded EN.

### Plugin Skeleton
- `zeitrak-core/src/plugin.rs`: `ZeitrakPlugin` trait (id/version/permissions) + `PluginRegistry::register`/`all_permissions`. **No runtime, no hooks, no Extism integration**. Will be replaced/extended by `zeitrak-plugin-host` in Phase 3 of the plan.

---

## 8. Prioritised Findings

| ID | Prio | Area | Finding | Recommendation | Effort |
|---|---|---|---|---|---|
| F1 | **P0** | Event store | No event upcasting / versioning. `schema_version` exists in the DDL but is never read. | `EventUpcaster` trait + dispatch in the repository load path. Prerequisite for plugin event evolution. | 1–2 days |
| F2 | **P1** | Event store | `Root::rehydrate_from_state(0, user)` bug — user reads ignore snapshot version. | Pull real version from snapshot repository. | 1–2 h |
| F3 | **P1** | Auth | Admin role name `"admin"` hardcoded and case-sensitive. | Env var `ADMIN_ROLE_NAME` OR dedicated `Permission::ADMIN_BYPASS`. Prerequisite for plugin trust levels. | 1–2 h |
| F4 | **P1** | Aggregate | Invitation lacks integration tests. | Test module following the pattern used for other aggregates. | 2–4 h |
| F5 | **P1** | Aggregate | No unified snapshot strategy (`snapshot_every` per aggregate). | Add `snapshot_every` as aggregate metadata, derive policy. Models the same approach used for plugin aggregates. | 0.5–1 day |
| F6 | **P1** | Auth | No refresh-token flow for user sessions; 1-hour hard cut. | Add refresh endpoint OR explicitly accept + document the constraint. | 4–6 h |
| F7 | **P2** | Aggregate | User: no SoftDelete/Restore, no dedicated ChangePassword. | Add when the plugin platform requires it. | 0.5 day |
| F8 | **P2** | Aggregate | Workspace: no `Delete` command. | Decide soft vs hard delete, then add. | 0.5 day |
| F9 | **P2** | Aggregate | No unified `FilterExpr`/`Page`/`Sort` DSL. | Introduce `zeitrak-core/src/shared/query.rs`, migrate incrementally. | 1–2 days |
| F10 | **P2** | Auth | JWT `kid` header is not validated. | Defer until multi-key / rotation is required. | — |
| F11 | **P2** | Auth | List endpoints don't call `require_permission`. | Explicitly document the privacy model OR add `*.list` permissions. | 0–0.5 day |
| F12 | **P2** | Backend | No backend i18n on API errors. | Add when required (post-plugin). | — |
| F13 | **P2** | Tests | No snapshot/projection recovery tests, no multi-workspace concurrency tests. | Extend the test suite. | 1 day |
| F14 | **P2** | Repo hygiene | Remote branches `desktop`, `multi-user`, `old`, `refactor`, `refactoring` exist but are not tracked locally. | Audit whether they hold WIP that should be reconciled or pruned. | 1–2 h |

---

## 9. Consequences for the Plugin Plan

1. **Phase 2 shrinks dramatically**. The original assumption of a large CRUD completion does not hold. Instead:
   - **P0 prerequisite**: F1 (event upcasting) — critical for plugin-event schema evolution.
   - **P1 bundle**: F2, F3, F4, F5, F6 before or alongside the plugin platform.
   - **P2**: as needed, during or after the plugin platform work.

2. **Phase 7 (dioxus-extism)** stays unchanged in scope — the generic Manifest-Extensions / `call_plugin` / `HostCapability::Custom` / route-`Replace` are still required.

3. **Phase 3 (`zeitrak-plugin-host`)** benefits from the fact that the aggregate structures are already clean. Hook points can be inserted into the existing application services directly (`zeitrak/src/{admin,tenant}/*.rs`).

4. **Multi-tenancy guarantees** remain intact under the plugin system — plugin code runs in a sandbox; the storage-API wrapper must respect `Pool<ScopeTenant>` and never switch to `Pool<ScopeAdmin>` without a trust check.

5. **Snapshots & versioning**: `snapshot_every` in the plugin manifest (plan Phase 5) should be consistent with the resolution of F5 — one strategy for both core and plugins.
