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

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};
use sqlx::Row;
use tempfile::TempDir;
use url::Url;
use zeitrak_core::admin::user::UserRepository as UserRepositoryTrait;
use zeitrak_infrastructure::{
    database::{DatabaseUri, Migrate},
    email::EmailSender,
};
use zeitrak_infrastructure_impl::{
    Pool, ScopeAdmin, ScopeTenant, StateConnected,
    admin::user::{projectors::UserProjector, repositories::UserRepository},
};

#[cfg(feature = "postgres")]
static PG_BASE_URL: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

/// Returns the base `PostgreSQL` URL for test databases.
///
/// If `ZEITRAK_TEST_POSTGRES_URL` is set (e.g. `postgres://postgres:postgres@postgres:5432`),
/// that URL is used directly — useful in devcontainers or CI where a Postgres
/// instance is already running. Otherwise falls back to starting a testcontainer.
#[cfg(feature = "postgres")]
async fn pg_base_url() -> &'static str {
    use testcontainers::runners::AsyncRunner as _;

    PG_BASE_URL
        .get_or_init(|| async {
            if let Ok(url) = std::env::var("ZEITRAK_TEST_POSTGRES_URL") {
                return url;
            }

            let container = testcontainers_modules::postgres::Postgres::default()
                .start()
                .await
                .expect("Postgres container must start");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("must get Postgres port");
            Box::leak(Box::new(container));
            format!("postgres://postgres:postgres@127.0.0.1:{port}")
        })
        .await
}

// ── unique fixture counter ────────────────────────────────────────────────────

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_fixture_id() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ── TestFixture ───────────────────────────────────────────────────────────────

/// A pair of fully-migrated, isolated databases for tests.
///
/// With the `sqlite` feature (default for tests), each fixture creates two
/// `SQLite` files in a temporary directory.  With the `postgres` feature, each
/// fixture creates two freshly-named databases in a shared `Postgres` container
/// started via testcontainers.
///
/// Tests can run fully in parallel — each fixture has its own isolated schema.
pub struct TestFixture {
    /// Admin database — schema matches `zeitrak-admin-migrations`.
    pub admin: Pool<ScopeAdmin, StateConnected>,
    /// Tenant database — schema matches `zeitrak-tenant-migrations`.
    pub tenant: Pool<ScopeTenant, StateConnected>,
    // Keeps the SQLite temp dir alive. None when using Postgres.
    _dir: Option<TempDir>,
}

impl TestFixture {
    /// Creates a fresh, isolated `TestFixture`.
    ///
    /// With `sqlite` feature: creates a temp directory with two `SQLite` files
    /// and runs all migrations.  With `postgres` feature: creates two Postgres
    /// databases in a shared testcontainer and runs all migrations.
    ///
    /// # Panics
    ///
    /// Panics if the database cannot be opened or migrations fail.
    pub async fn setup() -> Self {
        dotenvy::from_filename_override(".env.test").ok();
        sqlx::any::install_default_drivers();

        #[cfg(feature = "postgres")]
        return Self::setup_postgres().await;

        #[cfg(not(feature = "postgres"))]
        Self::setup_sqlite().await
    }

    #[cfg(not(feature = "postgres"))]
    async fn setup_sqlite() -> Self {
        let id = next_fixture_id();
        let dir = tempfile::Builder::new()
            .prefix(&format!("zeitrak_test_{id}_"))
            .tempdir()
            .expect("must create temp directory for test fixture");

        let admin_path = dir.path().join("admin.db");
        let tenant_path = dir.path().join("tenant.db");

        // sqlx-sqlite defaults to create_if_missing=false.
        std::fs::File::create(&admin_path).expect("must create admin.db");
        std::fs::File::create(&tenant_path).expect("must create tenant.db");

        let admin_url = Url::parse(&format!("sqlite://{}", admin_path.display()))
            .expect("admin URL must parse");
        let tenant_url = Url::parse(&format!("sqlite://{}", tenant_path.display()))
            .expect("tenant URL must parse");

        let admin = Pool::connect(&DatabaseUri::from(admin_url))
            .await
            .unwrap_or_else(|e| panic!("could not open admin test DB: {e}"));
        admin
            .migrate_database()
            .await
            .expect("admin migrations must succeed in TestFixture::setup");

        let tenant = Pool::connect(&DatabaseUri::from(tenant_url))
            .await
            .unwrap_or_else(|e| panic!("could not open tenant test DB: {e}"));
        tenant
            .migrate_database()
            .await
            .expect("tenant migrations must succeed in TestFixture::setup");

        Self {
            admin,
            tenant,
            _dir: Some(dir),
        }
    }

    #[cfg(feature = "postgres")]
    async fn setup_postgres() -> Self {
        let base = pg_base_url().await;
        let id = next_fixture_id();
        let admin_db = format!("zeitrak_test_{id}_admin");
        let tenant_db = format!("zeitrak_test_{id}_tenant");

        // Connect to the default `postgres` database to issue CREATE DATABASE.
        let bootstrap = sqlx::AnyPool::connect(&format!("{base}/postgres"))
            .await
            .expect("must connect to default postgres database");

        // Drop leftover databases from previous runs, then create fresh ones.
        for db in [&admin_db, &tenant_db] {
            let drop = format!("DROP DATABASE IF EXISTS \"{db}\"");
            sqlx::query(sqlx::AssertSqlSafe(drop.as_str()))
                .execute(&bootstrap)
                .await
                .unwrap_or_else(|e| panic!("must drop test DB {db}: {e}"));

            let create = format!("CREATE DATABASE \"{db}\"");
            sqlx::query(sqlx::AssertSqlSafe(create.as_str()))
                .execute(&bootstrap)
                .await
                .unwrap_or_else(|e| panic!("must create test DB {db}: {e}"));
        }

        let admin_url = Url::parse(&format!("{base}/{admin_db}")).expect("admin URL must parse");
        let tenant_url = Url::parse(&format!("{base}/{tenant_db}")).expect("tenant URL must parse");

        let admin: Pool<ScopeAdmin, StateConnected> = Pool::connect(&DatabaseUri::from(admin_url))
            .await
            .unwrap_or_else(|e| panic!("could not open admin test DB: {e}"));
        admin
            .migrate_database()
            .await
            .expect("admin migrations must succeed in TestFixture::setup");

        let tenant: Pool<ScopeTenant, StateConnected> =
            Pool::connect(&DatabaseUri::from(tenant_url))
                .await
                .unwrap_or_else(|e| panic!("could not open tenant test DB: {e}"));
        tenant
            .migrate_database()
            .await
            .expect("tenant migrations must succeed in TestFixture::setup");

        // Databases are dropped automatically when the Postgres container exits.
        Self {
            admin,
            tenant,
            _dir: None,
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

// ── Projector helpers ─────────────────────────────────────────────────────────

/// Runs all events currently in the admin database through the [`UserProjector`].
///
/// Call this after application-service operations that save to the event store
/// (e.g. `register_user_on`) so that the projection tables reflect the latest
/// state before making read assertions.
///
/// In production the projector runs as a separate daemon; in tests this function
/// replaces it with a single synchronous flush.
///
/// # Panics
///
/// Panics if the event query or any projector handler returns an error.
pub async fn flush_user_projector(pool: &Pool<ScopeAdmin, StateConnected>) {
    let rows = sqlx::query(
        "SELECT event_stream_id, type, version, global_position, event, metadata, schema_version \
         FROM events ORDER BY global_position",
    )
    .fetch_all(pool.as_ref())
    .await
    .expect("must query events table");

    let mut projector = UserProjector::new(pool.clone());

    for row in rows {
        let stream_id: String = row.get("event_stream_id");
        let event_type: String = row.get("type");
        let version: i64 = row.get("version");
        let global_position: i64 = row.get("global_position");
        let payload_bytes: Vec<u8> = row.get("event");
        let metadata_bytes: Option<Vec<u8>> = row.try_get("metadata").ok();
        let metadata = metadata_bytes
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or(serde_json::Value::Null);
        let schema_version: i64 = row.get("schema_version");

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        projector
            .handle(RawEvent {
                stream_id,
                event_type,
                version,
                global_position,
                payload_bytes,
                metadata,
                schema_version: schema_version as u32,
            })
            .await
            .expect("UserProjector must handle event without error");
    }
}

// ── Projection query helpers ──────────────────────────────────────────────────

/// Returns `true` when the user's `is_verified` flag is set in the projection.
///
/// Call after [`flush_user_projector`] to assert the verified state that the
/// projector writes from `UserVerified` events.
///
/// # Panics
///
/// Panics if the repository cannot be created, the query fails, or no user row
/// exists for `user_id`.
pub async fn is_user_email_verified(
    pool: &Pool<ScopeAdmin, StateConnected>,
    user_id: &str,
) -> bool {
    let repo = UserRepository::from_pool(pool.clone())
        .await
        .expect("must create UserRepository");
    repo.find_view_by_id(user_id)
        .await
        .expect("find_view_by_id must succeed")
        .expect("user row must exist")
        .is_verified
}

// ── RecordingEmailSender ──────────────────────────────────────────────────────

/// A captured outbound email.
#[derive(Debug, Clone)]
pub struct SentEmail {
    pub to: String,
    pub kind: SentEmailKind,
}

/// The specific content of a captured email.
#[derive(Debug, Clone)]
pub enum SentEmailKind {
    Invitation {
        invitation_link: String,
        workspace_name: String,
        invited_by_name: String,
        ttl_days: u32,
    },
    Verification {
        verification_link: String,
    },
    WaitlistNotification {
        subscriber_email: String,
    },
    WaitlistConfirmation,
}

/// An [`EmailSender`] that records every outbound email in memory.
///
/// Use this in tests to assert that the right emails were sent without
/// requiring a real SMTP server.
///
/// ```rust,ignore
/// let sender = RecordingEmailSender::new();
/// // … call code under test that accepts &dyn EmailSender …
/// let sent = sender.sent();
/// assert_eq!(sent.len(), 1);
/// ```
#[derive(Clone, Default)]
pub struct RecordingEmailSender {
    sent: Arc<Mutex<Vec<SentEmail>>>,
}

impl RecordingEmailSender {
    /// Creates a new, empty recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of all emails sent so far.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn sent(&self) -> Vec<SentEmail> {
        self.sent
            .lock()
            .expect("mutex must not be poisoned")
            .clone()
    }
}

#[async_trait]
impl EmailSender for RecordingEmailSender {
    async fn send_invitation(
        &self,
        to: &str,
        invitation_link: &str,
        workspace_name: &str,
        invited_by_name: &str,
        ttl_days: u32,
    ) -> anyhow::Result<()> {
        self.sent
            .lock()
            .expect("mutex must not be poisoned")
            .push(SentEmail {
                to: to.to_string(),
                kind: SentEmailKind::Invitation {
                    invitation_link: invitation_link.to_string(),
                    workspace_name: workspace_name.to_string(),
                    invited_by_name: invited_by_name.to_string(),
                    ttl_days,
                },
            });
        Ok(())
    }

    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> anyhow::Result<()> {
        self.sent
            .lock()
            .expect("mutex must not be poisoned")
            .push(SentEmail {
                to: to.to_string(),
                kind: SentEmailKind::Verification {
                    verification_link: verification_link.to_string(),
                },
            });
        Ok(())
    }

    async fn send_waitlist_notification(
        &self,
        owner_email: &str,
        subscriber_email: &str,
    ) -> anyhow::Result<()> {
        self.sent
            .lock()
            .expect("mutex must not be poisoned")
            .push(SentEmail {
                to: owner_email.to_string(),
                kind: SentEmailKind::WaitlistNotification {
                    subscriber_email: subscriber_email.to_string(),
                },
            });
        Ok(())
    }

    async fn send_waitlist_confirmation(&self, to: &str) -> anyhow::Result<()> {
        self.sent
            .lock()
            .expect("mutex must not be poisoned")
            .push(SentEmail {
                to: to.to_string(),
                kind: SentEmailKind::WaitlistConfirmation,
            });
        Ok(())
    }
}
