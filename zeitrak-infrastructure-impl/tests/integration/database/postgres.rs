#[cfg(feature = "postgres")]
pub mod tests {
    use zeitrak_tests::TestFixture;

    /// Verifies that all admin and tenant migrations run successfully on a fresh
    /// `PostgreSQL` database via testcontainers.
    #[tokio::test]
    async fn test_setup_postgres_database() {
        let _db = TestFixture::setup().await;
    }

    /// Verify that the expected admin tables exist after migrations.
    #[tokio::test]
    async fn test_admin_tables_visible_after_setup() {
        let db = TestFixture::setup().await;
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name::text FROM information_schema.tables \
             WHERE table_schema = 'public' ORDER BY table_name",
        )
        .fetch_all(db.admin.as_ref())
        .await
        .expect("information_schema query must succeed");

        let names: Vec<&str> = rows.iter().map(|(n,)| n.as_str()).collect();

        let expected = [
            "event_streams",
            "events",
            "snapshots",
            "permissions",
            "projections__users",
            "projections__workspaces",
            "projections__workspace_roles",
            "projections__workspace_user_roles",
            "projections__invitations",
            "smtp_config",
            "waitlist_signups",
            "plugin_audit",
        ];

        for table in expected {
            assert!(
                names.contains(&table),
                "{table} must exist after admin setup, found: {names:?}"
            );
        }
    }

    /// Verify that the expected tenant tables exist after migrations.
    #[tokio::test]
    async fn test_tenant_tables_visible_after_setup() {
        let db = TestFixture::setup().await;
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name::text FROM information_schema.tables \
             WHERE table_schema = 'public' ORDER BY table_name",
        )
        .fetch_all(db.tenant.as_ref())
        .await
        .expect("information_schema query must succeed");

        let names: Vec<&str> = rows.iter().map(|(n,)| n.as_str()).collect();

        let expected = [
            "event_streams",
            "events",
            "snapshots",
            "projections__activities",
            "projections__timesheets",
            "projections__timesheet_tags",
            "projections__timesheet_has_tags",
        ];

        for table in expected {
            assert!(
                names.contains(&table),
                "{table} must exist after tenant setup, found: {names:?}"
            );
        }
    }

    /// Verify that UUID columns have the correct `PostgreSQL` `uuid` type (not text).
    #[tokio::test]
    async fn test_uuid_columns_have_correct_type() {
        let db = TestFixture::setup().await;

        let uuid_columns = [
            ("projections__users", "id"),
            ("projections__workspaces", "id"),
            ("projections__workspace_roles", "id"),
            ("projections__workspace_roles", "workspace_id"),
            ("projections__invitations", "id"),
            ("projections__invitations", "workspace_id"),
            ("projections__invitations", "invited_by"),
            ("projections__invitations", "workspace_role_id"),
        ];

        for (table, column) in uuid_columns {
            let row: (String,) = sqlx::query_as(
                "SELECT data_type::text FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1::text AND column_name = $2::text",
            )
            .bind(table)
            .bind(column)
            .fetch_one(db.admin.as_ref())
            .await
            .unwrap_or_else(|e| panic!("must find column {table}.{column}: {e}"));

            assert_eq!(
                row.0, "uuid",
                "{table}.{column} must be uuid type, got: {}",
                row.0
            );
        }
    }

    /// Verify that permission seeds ran correctly on `PostgreSQL`.
    #[tokio::test]
    async fn test_permissions_seeded() {
        let db = TestFixture::setup().await;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permissions")
            .fetch_one(db.admin.as_ref())
            .await
            .expect("must count permissions");

        assert!(
            count.0 > 0,
            "permissions table must be seeded after setup, found 0 rows"
        );
    }

    /// Verify that foreign key constraints are enforced on `PostgreSQL`.
    /// Inserting an invitation with a non-existent `workspace_id` must fail.
    #[tokio::test]
    async fn test_foreign_key_constraints_enforced() {
        let db = TestFixture::setup().await;

        let result = sqlx::query(
            "INSERT INTO projections__invitations \
                 (id, workspace_id, invited_by, email, workspace_role_id, token, status, expires_at) \
             VALUES \
                 ('00000000-0000-0000-0000-000000000001'::uuid, \
                  '00000000-0000-0000-0000-000000000099'::uuid, \
                  '00000000-0000-0000-0000-000000000099'::uuid, \
                  'test@example.com', \
                  '00000000-0000-0000-0000-000000000099'::uuid, \
                  'tok_test', 'pending', '2099-01-01T00:00:00Z')",
        )
        .execute(db.admin.as_ref())
        .await;

        assert!(
            result.is_err(),
            "INSERT with non-existent `workspace_id` must fail due to FK constraint"
        );
    }

    /// Verify that UUID round-trip works: insert a row with UUID values, then
    /// read it back and confirm the values match.
    #[tokio::test]
    async fn test_uuid_round_trip() {
        let db = TestFixture::setup().await;

        let workspace_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c91";
        sqlx::query("INSERT INTO projections__workspaces (id, name) VALUES ($1::uuid, $2)")
            .bind(workspace_id)
            .bind("Test Workspace")
            .execute(db.admin.as_ref())
            .await
            .expect("must insert workspace");

        let row: (String,) =
            sqlx::query_as("SELECT id::text FROM projections__workspaces WHERE id = $1::uuid")
                .bind(workspace_id)
                .fetch_one(db.admin.as_ref())
                .await
                .expect("must read workspace back");

        assert_eq!(row.0, workspace_id);
    }

    /// Verify that tenant UUID columns also have the correct type.
    #[tokio::test]
    async fn test_tenant_uuid_columns_have_correct_type() {
        let db = TestFixture::setup().await;

        let uuid_columns = [
            ("projections__activities", "id"),
            ("projections__timesheets", "id"),
            ("projections__timesheets", "user_id"),
            ("projections__timesheet_tags", "id"),
            ("projections__timesheet_has_tags", "timesheet_id"),
            ("projections__timesheet_has_tags", "timesheet_tag_id"),
        ];

        for (table, column) in uuid_columns {
            let row: (String,) = sqlx::query_as(
                "SELECT data_type::text FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1::text AND column_name = $2::text",
            )
            .bind(table)
            .bind(column)
            .fetch_one(db.tenant.as_ref())
            .await
            .unwrap_or_else(|e| panic!("must find column {table}.{column}: {e}"));

            assert_eq!(
                row.0, "uuid",
                "{table}.{column} must be uuid type, got: {}",
                row.0
            );
        }
    }
}
