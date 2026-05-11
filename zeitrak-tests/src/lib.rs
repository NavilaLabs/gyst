//! Shared test infrastructure for the Zeitrak workspace.
//!
//! # The modern way: `TestFixture`
//!
//! [`TestFixture`] gives every test a pair of fully-migrated, isolated
//! **file-based `SQLite` databases** in a **temporary directory**.  Because each
//! fixture gets a unique temp directory:
//!
//! * tests run **fully in parallel** — no `#[serial]` required
//! * databases are **automatically cleaned up** when the fixture is dropped
//! * every test starts from a **known-empty state**
//!
//! ## Why not named in-memory `SQLite`?
//!
//! `sqlite:///file:name?mode=memory&cache=shared` does not work as expected
//! with sqlx's `AnyPool`: each connection acquires its own private anonymous
//! in-memory database instead of sharing the named one.  The `SeaORM` migrator
//! always opens a second connection, so its changes are invisible to the
//! pool's connection.  Temp-directory `SQLite` avoids this entirely: the
//! `SeaORM` migrator and the pool both connect to the same on-disk file.
//!
//! ```rust,ignore
//! use zeitrak_tests::TestFixture;
//!
//! #[tokio::test]
//! async fn it_works() {
//!     let db = TestFixture::setup().await;
//!     // db.admin  — Pool<ScopeAdmin, StateConnected>  (admin migrations applied)
//!     // db.tenant — Pool<ScopeTenant, StateConnected> (tenant migrations applied)
//! }
//! ```
//!
//! # Lifecycle hooks (for env-only tests)
//!
//! [`test_lifecycle`] and [`test_database_lifecycle`] are kept for the small
//! number of tests (e.g. JWT validation) that need `.env.test` loaded but do
//! **not** need a database.  Database tests should use [`TestFixture`] instead.

use std::sync::atomic::{AtomicU64, Ordering};

use tempfile::TempDir;
use url::Url;
use zeitrak_infrastructure::database::{DatabaseUri, Migrate};
use zeitrak_infrastructure_impl::{Error, Pool, ScopeAdmin, ScopeTenant, StateConnected};

// ── unique fixture counter ────────────────────────────────────────────────────

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_fixture_id() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ── TestFixture ───────────────────────────────────────────────────────────────

/// A pair of fully-migrated, isolated `SQLite` databases in a temp directory.
///
/// Each call to [`TestFixture::setup`] creates a fresh temporary directory and
/// opens two `SQLite` files inside it (`admin.db` and `tenant.db`), running all
/// migrations on each.  Because each fixture has its own directory, tests can
/// run concurrently without any shared state.
///
/// The temporary directory is automatically deleted when the fixture is dropped.
pub struct TestFixture {
    /// Admin database — schema matches `zeitrak-admin-migrations`.
    pub admin: Pool<ScopeAdmin, StateConnected>,
    /// Tenant database — schema matches `zeitrak-tenant-migrations`.
    pub tenant: Pool<ScopeTenant, StateConnected>,
    // Keeps the temp dir alive for the lifetime of the fixture.
    _dir: TempDir,
}

impl TestFixture {
    /// Creates a fresh, isolated `TestFixture`.
    ///
    /// Loads `.env.test` (safe to call from multiple parallel tests — all
    /// tests load the same values), installs `SQLx` any-DB drivers, creates
    /// a temporary directory with two `SQLite` databases, and runs all
    /// migrations on them.
    ///
    /// # Panics
    ///
    /// Panics if the temp directory cannot be created, if a database cannot
    /// be opened, or if migrations fail.  In a test context this always
    /// indicates a programming error.
    pub async fn setup() -> Self {
        // Load the test environment so CONFIG is initialised correctly.
        dotenvy::from_filename_override(".env.test").ok();
        sqlx::any::install_default_drivers();

        let id = next_fixture_id();
        let dir = tempfile::Builder::new()
            .prefix(&format!("zeitrak_test_{id}_"))
            .tempdir()
            .expect("must create temp directory for test fixture");

        let admin_path = dir.path().join("admin.db");
        let tenant_path = dir.path().join("tenant.db");

        // sqlx-sqlite defaults to create_if_missing=false, so we must create
        // the files before opening the pool connections.
        std::fs::File::create(&admin_path).expect("must create admin.db");
        std::fs::File::create(&tenant_path).expect("must create tenant.db");

        let admin_url = Url::parse(&format!("sqlite://{}", admin_path.display()))
            .expect("admin URL must parse");
        let tenant_url = Url::parse(&format!("sqlite://{}", tenant_path.display()))
            .expect("tenant URL must parse");

        let admin = Pool::connect(&DatabaseUri::from(admin_url))
            .await
            .unwrap_or_else(|e: Error| panic!("could not open admin test DB: {e}"));
        admin
            .migrate_database()
            .await
            .expect("admin migrations must succeed in TestFixture::setup");

        let tenant = Pool::connect(&DatabaseUri::from(tenant_url))
            .await
            .unwrap_or_else(|e: Error| panic!("could not open tenant test DB: {e}"));
        tenant
            .migrate_database()
            .await
            .expect("tenant migrations must succeed in TestFixture::setup");

        Self {
            admin,
            tenant,
            _dir: dir,
        }
    }
}

// ── lifecycle hooks (for env-only tests) ─────────────────────────────────────

/// Lifecycle hooks for tests that need `.env.test` loaded but do **not** need
/// a database (e.g. JWT token validation).
///
/// Database tests should use [`TestFixture`] instead.
pub mod test_lifecycle {
    /// # Panics
    ///
    /// Panics if `.env.test` cannot be loaded.
    pub fn before() {
        dotenvy::from_filename_override(".env.test").expect("Failed to load .env.test.");
    }

    pub fn after() {
        dotenvy::from_filename_override(".env").ok();
    }
}

/// Like [`test_lifecycle`] but also installs the `SQLx` any-DB drivers.
///
/// Needed by tests that use the global `Pool::connect_*()` methods (driven by
/// `CONFIG`) rather than [`TestFixture`] — for example, the Postgres
/// integration tests that talk to a real container.
pub mod test_database_lifecycle {
    use sqlx::any::install_default_drivers;

    use crate::test_lifecycle;

    pub fn before() {
        test_lifecycle::before();
        install_default_drivers();
    }

    pub fn after() {
        test_lifecycle::after();
    }
}
