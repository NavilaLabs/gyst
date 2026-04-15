# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Backend (Cargo workspace root: `/workspaces/loom`)

```bash
cargo build                          # Build all workspace members
cargo test --all                     # Run all tests
cargo test -p <package>              # Test a single package
cargo test <test_name>               # Run a single test by name
cargo clippy --all-targets --all-features -- -D warnings  # Lint (matches pre-commit)
cargo fmt --all                      # Format code
cargo fmt --all -- --check           # Check formatting without modifying
```

### Migrations (run from workspace root)

```bash
# Generate a new migration (interactive: asks for scope and name)
./scripts/new-migration.sh

# Run migrations manually for a specific scope (admin or tenant)
cargo run -p loom-admin-migrations -- up
cargo run -p loom-tenant-migrations -- up
cargo run -p loom-admin-migrations -- fresh   # Drop and re-run all
cargo run -p loom-admin-migrations -- down    # Roll back one migration
```

### Frontend (Dioxus — from `/workspaces/loom/loom-presentation/gui`)

```bash
dx serve --addr 0.0.0.0 --port 8080  # Serve web app (default, port 8080)
dx serve --platform desktop          # Serve desktop app
dx build --release                   # Build for release
```

### Projection daemons (run from workspace root)

```bash
cargo run -p loom --bin admin-projection-daemon   # Admin projections (users, workspaces, roles, permissions)
cargo run -p loom --bin tenant-projection-daemon  # Tenant projections (customers, projects, activities, timesheets, tags, rates)
                                                  # Discovers all workspaces from the admin DB automatically
```

### Dev utilities

```bash
./scripts/start-dev-tw.sh            # Start Tailwind CSS watcher
./scripts/start-dev-web.sh           # Start web dev server
./scripts/add-component.sh           # Scaffold a new UI component
./scripts/new-scaffold.sh            # Generate new code scaffold
```

## Architecture Overview

This is a **multi-tenant SaaS application** using Event Sourcing + CQRS + Domain-Driven Design (DDD), implemented as a Rust Cargo workspace.

### Workspace Structure

```
loom/                          # Main integration crate (wires everything together)
loom-core/                     # Pure domain logic — no I/O, no framework deps
loom-infrastructure/           # Trait abstractions (repository interfaces, etc.)
loom-infrastructure-impl/      # Concrete implementations (SQLx/SeaORM databases)
loom-migrations/
  loom-admin-migrations/       # Schema for the shared admin database
  loom-shared-migrations/      # Shared migration utilities
  loom-tenant-migrations/      # Schema for per-tenant databases
loom-tests/                    # Shared test helpers (DB setup/teardown)
loom-presentation/gui/         # Separate Cargo workspace (Dioxus UI)
  packages/api/                # Server functions + business logic bridge
  packages/ui/                 # Shared UI components (cross-platform)
  packages/web/                # Web SPA target
  packages/desktop/            # Desktop target
  packages/mobile/             # Mobile target
with-lifecycle/                # Proc macro for aggregate lifecycle management
```

### Domain Layer (`loom-core`)

Code is organized by **bounded context** → **aggregate** → **layer**:

```
src/
  admin/user/
    domain/       # Aggregate, events, value objects, repository interfaces
    application/  # Commands, queries, views, projectors
  tenant/         # (future tenant-scoped domains)
  shared/         # Cross-domain types (AggregateId, etc.)
```

Each domain aggregate follows this pattern:
- `domain/aggregates.rs` — aggregate state + event application logic
- `domain/events.rs` — all domain events as an enum
- `domain/interfaces.rs` — repository trait definitions
- `application/commands.rs` — command handlers
- `application/queries.rs` — query handlers
- `application/views.rs` — read model / view types

### Multi-Tenant Database Strategy

- **Admin database** (`loom_admin`) — single shared database for system-wide operations
- **Tenant databases** (`loom_tenant_{token}`) — one isolated database per tenant. A tenant is also refered as a workspace in the codebase and the ui

Both use the same event store schema:
- `event_streams` — stream metadata
- `events` — append-only event log with global position
- `snapshots` — periodic snapshots for read performance
- Projection tables (e.g., `projections__users`)

### Event Sourcing

Uses `eventually-rs`. The flow is:
1. Command → command handler validates → produces `DomainEvent`
2. Events appended to event store via repository
3. Projectors consume events → update read model tables
4. Queries read from projection tables

A projection daemon (`loom-presentation/gui/packages/api`) runs as a background task to keep projections up to date.

### Environment & Configuration

Copy `.env.dev.dist` to `.env.dev` and `.env.test.dist` to `.env.test` to get started. Key env vars:

```
DATABASE_BASE_URI=sqlite:///workspaces/loom/.devcontainer/database
ADMIN_DATABASE_NAME=loom_admin
```

YAML config files live in `config/development/` (application, database, logging settings).

### Frontend (`loom-presentation/gui`)

A **separate Cargo workspace** using [Dioxus 0.7](https://dioxuslabs.com/learn/0.7). Important Dioxus 0.7 notes (see `AGENTS.md` in that directory):
- `cx`, `Scope`, and `use_state` are **removed** in 0.7
- Use `use_signal` for local state, `use_resource` for async, `use_context_provider`/`use_context` for shared state
- Server functions use `#[post]`/`#[get]` macros
- Components require `#[component]` macro; function name must start with uppercase or contain `_`

In the `assets` directory there are mockups of what I expect the application is supposed to look like.

### Code Quality

Clippy is configured at `deny` level for `all`, `correctness`, `complexity`, `perf`, and `style`, with `warn` for `pedantic` and `nursery`. Pre-commit hooks enforce `cargo fmt`, `cargo clippy`, and `cargo test` (test runs on push only).

### Testing

**Domain unit tests** live directly in the source files under `#[cfg(test)]`:
- `loom-core/src/admin/user/domain/aggregates.rs` — `User::apply` tests
- `loom-core/src/admin/user/application/commands.rs` — `UserCommand::create` tests

**Integration tests** live in `loom-infrastructure-impl/tests/integration/`:
- `database/sqlite.rs`, `database/postgres.rs` — DB setup and migration smoke tests
- `user/mod.rs` — repository save/get, projector event handling

The `loom-tests` crate provides helpers (`get_admin_pool`, `get_default_pool`, `refresh_databases`) for spinning up and tearing down admin/tenant databases. Tests use the lifecycle macros from `with-lifecycle`:

```rust
test_lifecycle::before()  // loads .env.test
test_lifecycle::after()   // resets to .env.dev
```

Integration tests that touch shared SQLite files must be annotated with `#[serial]` (from `serial_test`) to prevent parallel execution races. Add `#[serial]` before `#[with_lifecycle(...)]` on any test that calls `refresh_databases`.

Migrations must use `manager.database_backend()` (runtime detection) rather than `#[cfg(feature = "...")]` guards when the logic differs between SQLite and Postgres, because the integration test binary is compiled with only the `sqlite` feature by default.
