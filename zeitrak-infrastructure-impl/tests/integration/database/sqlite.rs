#[cfg(not(feature = "postgres"))]
pub mod tests {
    use zeitrak_tests::TestFixture;

    /// Verifies that admin and tenant migrations run successfully on a fresh
    /// isolated database.  If any migration panics or fails, the test fails.
    #[tokio::test]
    async fn test_setup_sqlite_database() {
        // TestFixture::setup runs all migrations; if any fail it panics.
        let _db = TestFixture::setup().await;
    }

    /// Verify that the expected tables are visible in the admin pool AFTER setup.
    ///
    /// Guards against a subtle failure mode: if migrations run on a different
    /// connection/database than the pool (e.g. with named in-memory `SQLite`),
    /// the pool's database would be empty even though migrations "succeeded".
    #[tokio::test]
    async fn test_tables_visible_in_admin_pool() {
        let db = TestFixture::setup().await;
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(db.admin.as_ref())
                .await
                .expect("sqlite_master query must succeed");
        let names: Vec<&str> = rows.iter().map(|(n,)| n.as_str()).collect();
        assert!(
            names.contains(&"event_streams"),
            "event_streams must exist after setup, found: {names:?}"
        );
        assert!(
            names.contains(&"projections__users"),
            "projections__users must exist after setup, found: {names:?}"
        );
    }
}
