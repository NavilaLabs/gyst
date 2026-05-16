# CLAUDE.md — Zeitrak

This file is the authoritative reference for Claude when working on the Zeitrak codebase. Read it in full before writing or reviewing any code.

---

## What is Zeitrak?

Zeitrak is a **local-first, cross-platform activity tracking application** built in Rust. It allows users to record how long they spend on activities — for example, "I worked from 08:00 to 16:00" or "I did sport from 08:00 to 10:00".

Primary targets are **iOS, Android, and Web**. Desktop (Linux/macOS/Windows) is supported via Dioxus but is not the primary focus.

The core business logic lives in a standalone library (`zeitrak-core`) that is completely UI-agnostic. The Dioxus-based GUI is a separate workspace at `zeitrak-presentation/gui`.

---

## Common Commands

### Backend (Cargo workspace root: `/workspaces/zeitrak`)

```bash
cargo build                          # Build all workspace members
cargo test --all                     # Run all tests
cargo test -p <package>              # Test a single package
cargo test <test_name>               # Run a single test by name
cargo clippy --all-targets --all-features -- -D warnings  # Lint (matches pre-commit)
cargo fmt --all                      # Format code
cargo fmt --all -- --check           # Check formatting without modifying
```

---

## Repository Structure

```
zeitrak/                          # Root Cargo workspace
├── CLAUDE.md                     # This file
├── ROADMAP.md                    # Planned features (non-binding)
├── Cargo.toml                    # Workspace manifest
├── zeitrak-core/                 # Domain logic — no I/O, no frameworks
├── zeitrak-infrastructure/       # Ports (traits): config, database contracts
├── zeitrak-infrastructure-impl/  # Adapters: SQLx, SeaQuery, SeaORM migrations
├── zeitrak-migrations/
│   ├── zeitrak-shared-migrations/   # Shared event-store DDL (event_streams, events, snapshots)
│   ├── zeitrak-admin-migrations/    # Admin DB schema + seeds
│   └── zeitrak-tenant-migrations/   # Tenant (workspace) DB schema
├── zeitrak-tests/                # Shared test infrastructure (TestFixture, lifecycle hooks)
├── zeitrak/                      # Thin facade crate (application services, binary entry points)
│   └── src/bin/                  # Projection daemons, CLI tools
├── zeitrak-presentation/
│   └── gui/                      # Separate Cargo workspace (Dioxus)
│       ├── packages/api/         # Fullstack server functions
│       ├── packages/ui/          # Shared Dioxus components
│       ├── packages/web/         # Web platform entry point
│       ├── packages/desktop/     # Desktop platform entry point
│       └── packages/mobile/      # iOS/Android platform entry point
└── with-lifecycle/               # Proc-macro crate for test setup/teardown
```

---

## Architecture: The Onion Pattern

Zeitrak strictly follows the **Onion Architecture** (also known as Ports & Adapters / Hexagonal Architecture). The dependency rule is non-negotiable:

```
zeitrak-core          ← innermost, zero external I/O dependencies
zeitrak-infrastructure ← port traits (interfaces)
zeitrak-infrastructure-impl ← adapters (SQLx, SeaQuery, etc.)
zeitrak               ← application facade, wires everything together
zeitrak-presentation  ← UI layer, depends only on the facade
```

**Never violate the dependency direction.** `zeitrak-core` must not import from `zeitrak-infrastructure` or any I/O crate. `zeitrak-infrastructure` must not import from `zeitrak-infrastructure-impl`.

---

## Architecture: CQRS + Event Sourcing

Zeitrak uses **CQRS** (Command Query Responsibility Segregation) and **Event Sourcing** throughout. Understand these rules before touching any domain code:

### Writes — Event Store

All state changes are persisted as immutable events in the event store (`event_streams` + `events` tables). The crates used are:

- [`eventually-rs`](https://github.com/get-eventually/eventually-rs) — core `Aggregate`, `Root`, `Message` traits
- `eventually-any` — SQLx-backed event store + snapshot repository (**maintained by NavilaLabs, do not modify the dependency contract**)
- `eventually-projection` — projection runner, `Projector` trait, `SqlCheckpoint` (**maintained by NavilaLabs, do not modify the dependency contract**)

Every aggregate must implement `eventually::aggregate::Aggregate`. State transitions happen exclusively via events applied in `Aggregate::apply`. Commands are handled in the `application/commands.rs` layer and produce events via `Root::record_new` or `root.record_that(...)`.

### Reads — Projections

Never read from the event store for queries. All reads go through **projection tables** (e.g. `projections__users`, `projections__activities`). Projectors listen to the event stream and maintain these read models.

Projection tables are queried via the `ReadRepository` trait in `zeitrak-infrastructure`, implemented with `sea-query` + `sqlx` in `zeitrak-infrastructure-impl`.

### Snapshots

`eventually-any` provides snapshot support. Use it for aggregates that accumulate many events.

---

## Architecture: Multi-Tenancy

Zeitrak uses a **two-database multi-tenancy model**:

| Database | Purpose |
|---|---|
| **Admin DB** | Users, workspaces, roles, permissions, workspace-role assignments |
| **Tenant DB (per workspace)** | Business aggregates: activities, timesheets, workspace-roles, tags |

There is one admin database and one tenant database per workspace. On SQLite, these are separate `.sqlite` files. On PostgreSQL, they are separate databases.

The `Pool<Scope, State>` type in `zeitrak-infrastructure-impl` encodes the scope at the type level:
- `Pool<ScopeAdmin, StateConnected>` — admin pool
- `Pool<ScopeTenant, StateConnected>` — tenant pool
- `Pool<ScopeDefault, StateConnected>` — bootstrap pool (used only for DB initialization)

**Never use the wrong pool scope.** Pass the correct pool type explicitly; do not guess.

---

## Architecture: Database Abstraction

Zeitrak supports both **SQLite** (local use) and **PostgreSQL** (server/team use). Database-specific code is isolated to `zeitrak-infrastructure-impl`.

- **Queries** are built with [`sea-query`](https://github.com/SeaQL/sea-query) — never write raw SQL strings for production queries in repository code.
- **Execution** uses `sqlx` with the `AnyPool`/`AnyDriver` backend.
- **Migrations** use `sea-orm-migration` (in the `zeitrak-migrations/*` crates).
- The `DatabaseType` enum (`Sqlite` / `Postgres`) is available on `Pool` — use `pool.database_type()` to branch when a query truly cannot be made backend-agnostic.

Raw `sqlx::query(...)` strings are acceptable **only** in:
1. Migration files
2. Projector handlers (for simple `INSERT`/`UPDATE`/`DELETE` where sea-query would add no clarity)
3. Tests

---

## Domain Structure inside `zeitrak-core`

Every aggregate follows this layout (see `zeitrak-core/src/admin/user/` as the canonical example):

```
<domain>/
├── mod.rs               # Re-exports, domain-level Error enum
├── application/
│   ├── mod.rs           # aggregate_root macro, re-exports
│   ├── commands.rs      # Command trait + command handler struct (holds a repository)
│   ├── queries.rs       # Query structs (hold a repository + optional authenticator)
│   ├── rows.rs          # Read model row structs (returned from projections)
│   └── inputs.rs        # Validated input structs using `validator`
└── domain/
    ├── mod.rs
    ├── aggregates.rs    # Aggregate struct + `impl Aggregate`
    ├── events.rs        # Event enum + `impl Message`
    └── interfaces.rs    # Repository trait (extends ReadRepository + WriteRepository)
```

### Key rules:
- The **domain layer** (`domain/`) has zero knowledge of infrastructure. No `sqlx`, no `sea-query`, no I/O.
- The **application layer** (`application/`) holds command and query handlers that accept repository traits — never concrete implementations.
- Repository traits live in `domain/interfaces.rs` and extend `ReadRepository<A>` + `WriteRepository<A>` from `zeitrak-infrastructure`.
- Input structs in `inputs.rs` derive `validator::Validate` — always validate before dispatching a command.
- Error enums in `mod.rs` use `thiserror::Error`.

---

## The Repository Pattern

`zeitrak-infrastructure` defines:

```rust
pub trait ReadRepository<T: Aggregate> { ... }
pub trait WriteRepository<T: Aggregate> { ... }
```

Concrete implementations live in `zeitrak-infrastructure-impl/src/sea_query_sqlx/<scope>/<domain>/repositories.rs`.

**Rules:**
- Domain code only ever sees the repository trait, never the concrete struct.
- `ReadRepository` implementations may use `sea-query` to query projection tables.
- `WriteRepository` implementations delegate to `eventually-any`'s `Repository` (event store + snapshots).
- Never put SQL directly in `zeitrak-core`.

---

## Error Handling

| Context | Crate |
|---|---|
| Library/domain errors | `thiserror` |
| Application/binary errors | `anyhow` |

Rules:
- Every public `Error` enum must derive `thiserror::Error` and `Debug`.
- Never use `.unwrap()` in library code. Use `.expect("reason")` only in `main`, tests, or demonstrably-infallible paths — and always include an explanatory message.
- Never use `Box<dyn Error>` in public APIs. Use concrete error types or `thiserror` enums.
- Application service functions in `zeitrak/src/` return `anyhow::Result<T>`.
- The `ValidationError` newtype in `zeitrak/src/error.rs` is the standard way to signal domain input validation failures to the presentation layer (maps to HTTP 422).

---

## Form Validation

Use the [`validator`](https://crates.io/crates/validator) crate for all input validation.

- Input structs live in `application/inputs.rs` and derive `#[derive(Clone, Validate)]`.
- Always call `input.validate()?` before dispatching any command.
- Use `zeitrak_core::validation::validation_summary(&errors)` to produce a human-readable error message.
- The helper `zeitrak::error::validate(input)` wraps this into a single `anyhow::Result<()>` call for use in application services.

---

## Rust Code Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

- `snake_case` for functions, variables, modules
- `CamelCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- All public items must have `///` doc comments
- `#[derive(Debug)]` on every public struct and enum
- `#[non_exhaustive]` on public enums in library crates
- Prefer `impl Into<T>` over `T` in constructor arguments
- Prefer iterator chains over imperative loops for transformations
- Never clone to avoid a borrow checker fight — fix the design

### Clippy

The workspace enforces strict Clippy lints. All code must pass without warnings:

```toml
[lints.clippy]
all = { level = "deny" }
correctness = { level = "deny" }
complexity = { level = "deny" }
perf = { level = "deny" }
style = { level = "deny" }
pedantic = { level = "warn" }
nursery = { level = "warn" }
```

---

## Async

- `tokio` is the async runtime throughout.
- Use `async-trait` for async methods in traits (until stable `async fn in trait` is sufficient).
- Never block inside an async context. Use `tokio::task::spawn_blocking` for CPU-bound work.
- Prefer channels over `Arc<Mutex<T>>` for shared mutable state.

---

## Testing

### TestFixture

All integration tests must use `TestFixture` from `zeitrak-tests`:

```rust
use zeitrak_tests::TestFixture;

#[tokio::test]
async fn it_works() {
    let db = TestFixture::setup().await;
    // db.admin  — Pool<ScopeAdmin, StateConnected>  (all admin migrations applied)
    // db.tenant — Pool<ScopeTenant, StateConnected> (all tenant migrations applied)
}
```

`TestFixture::setup()` creates a fresh temporary directory with two isolated SQLite files and runs all migrations. Each test gets its own fixture — tests are fully parallel, no `#[serial]` needed.

**Why file-based SQLite and not named in-memory?** SeaORM's migrator always opens a second connection; named in-memory SQLite databases are not shared between connections in `sqlx`'s `AnyPool`. File-based SQLite in a temp directory solves this cleanly.

### Unit Tests

Domain logic (aggregates, events, application commands) is tested with pure unit tests in `#[cfg(test)] mod tests` blocks at the bottom of the relevant file. No database needed.

### Lifecycle hooks

For tests that only need `.env.test` loaded (e.g. JWT tests), use:

```rust
use zeitrak_tests::test_lifecycle;
use with_lifecycle::with_lifecycle;

#[with_lifecycle(test_lifecycle)]
#[test]
fn my_test() { ... }
```

### Language

All test names, assertion messages, and comments must be in **English**.

---

## Dioxus GUI (`zeitrak-presentation/gui`)

This is a separate Cargo workspace. Read `zeitrak-presentation/gui/AGENTS.md` for Dioxus-specific guidance.

### Component conventions

- Components are functions annotated with `#[component]`, named in `CamelCase`.
- Props must implement `PartialEq` and `Clone`.
- Use `use_signal` for local state, `use_context` / `use_context_provider` for shared state.
- Use `use_resource` for async data fetching (not `use_state` + manual effects).
- On the server side, use `use_server_future` for SSR-compatible async fetches.
- Never use `cx`, `Scope`, or `use_state` — these are Dioxus 0.4-era APIs.

### Server Functions

- Server functions are defined in `packages/api/` with `#[get]` / `#[post]` macros.
- Server-only dependencies are gated behind the `server` feature.
- Session extraction helpers live in `packages/api/src/session.rs` — always use these, never extract the session manually in a server function.
- Permissions are checked via `session::require_permission(&user, permissions::SOME_PERMISSION).await?` before any mutating operation.

### Styling

- Design tokens are defined in `packages/ui/assets/theme.css` using CSS custom properties.
- Tailwind CSS (v4) is used for utility classes — run `deno task tailwind` to regenerate.
- Component-scoped styles use `asset!("./style.css")` with `document::Link`.
- Never use inline `style="..."` attributes for anything beyond dynamic CSS variable overrides.
- The `GlobalStyles` component in `packages/ui/src/lib.rs` preloads all component stylesheets at the root — add new stylesheets there to prevent flash-of-unstyled-content.

### State Contexts (GUI)

| Type alias | Purpose |
|---|---|
| `RunningTimer` | Currently running timesheet DTO |
| `RunningElapsed` | Elapsed seconds for the running timer |
| `UserSettings` | Current user's display settings |
| `WorkspaceSettings` | Current workspace settings |
| `ActivitiesCache` | Pre-populated list of activities |
| `TagsCache` | Pre-populated list of tags |
| `TimesheetsCache` | Pre-populated recent timesheets |
| `SidebarOpen` | Sidebar open/collapsed state |

All contexts are provided in the top-level `Layout` component. Access them with `use_context::<T>()`.

---

## Authentication & Authorization

- Authentication uses **email + password** verified with `bcrypt`, then issues a **JWT** (HS256, 1-hour lifetime).
- JWT validation lives in `zeitrak/src/authentication.rs` — only HS256 is accepted, `alg:none` is always rejected.
- The `AuthorizationService` in `zeitrak/src/authorization.rs` provides live permission checks against projection tables. Always use the `_on(pool, ...)` variants in tests.
- Admin users (any user with a workspace role named `"admin"`) bypass individual permission checks — the role name match is **case-sensitive**.
- Never hardcode permission strings — use the constants in `zeitrak-core/src/permissions.rs`.

---

## Branching & Commits

### Branch naming

```
feature/<short-description>
fix/<short-description>
refactor/<short-description>
chore/<short-description>
docs/<short-description>
```

### Commit messages (Conventional Commits)

```
<type>(<scope>): <short summary in imperative mood>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `perf`, `ci`

Examples:
```
feat(core): add user invitation aggregate and events
fix(auth): reject alg:none tokens in JWT validation
refactor(infrastructure): extract SnapshotRepository generic wrapper
test(authorization): add cross-workspace isolation tests
```

- Summary line ≤ 72 characters
- Body explains *why*, not *what*
- All text in **English**

---

## What Claude Must Not Do

- Do not add SQL to `zeitrak-core`.
- Do not read from the event store for queries — use projection tables.
- Do not break the dependency direction (core ← infrastructure ← impl ← facade ← UI).
- Do not modify the public API of `eventually-any` or `eventually-projection` — these are external dependencies maintained separately.
- Do not use `unwrap()` in library code without an explanatory message.
- Do not use `Box<dyn Error>` in public API return types.
- Do not write comments or variable names in any language other than English.
- Do not use deprecated Dioxus APIs (`cx`, `Scope`, `use_state`).
- Do not use raw SQL strings in repository implementations — use `sea-query`.
- Do not hardcode permission strings — use the constants from `zeitrak-core::permissions`.
- Do not add `#[serial]` to integration tests — fix isolation instead by using `TestFixture`.
- Do not commit personal data (email addresses, names, credentials) anywhere in the repository — this is a public repo. Sensitive values belong in environment variables only.

---

## Planned Features (Not Yet Implemented)

The following are planned but not yet in the codebase. Do not assume they exist:

- **Extism plugin system** — runtime extensibility via WASM plugins (client-side and server-side plugin split). Architecture is not yet defined.
- **User invitation flow** — inviting users to a workspace via email.
- **Multi-user workspace access** — currently only the admin user can track activities.
- **Remote workspace sync** — local SQLite workspace linked to a remote PostgreSQL workspace for offline-first mobile use.
- **RBAC enforcement across all endpoints** — role/permission checks are partially in place but not complete.
