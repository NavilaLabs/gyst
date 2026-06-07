use sea_orm_migration::prelude::*;

/// Introduces `is_instance_admin` on the users projection and migrates existing
/// instances to the new admin model.
///
/// What this migration does:
///
/// 1. Adds `is_instance_admin BOOLEAN NOT NULL DEFAULT FALSE` to `projections__users`.
/// 2. Creates a partial unique index to enforce at most one instance admin.
/// 3. Removes the `admin.bypass` permission and all role/user grants that
///    reference it — instance admin status is now carried by the column, not
///    a permission.
/// 4. For every existing workspace that still has an "admin" role (legacy):
///    - Renames it to "workspace_admin" in the projection.
///    - Grants all 26 workspace-scoped permissions to it.
/// 5. Identifies the instance admin: the user with the smallest UUID (v7,
///    time-ordered) among users in any "workspace_admin" role — i.e. the one
///    who created the first workspace. Sets their `is_instance_admin = true`.
#[derive(DeriveMigrationName)]
pub struct Migration;

const ADMIN_BYPASS_ID: &str = "01100000-0000-7000-8000-000000000011";

/// UUIDs for all 26 workspace-scoped permissions.
const WORKSPACE_PERM_IDS: &[&str] = &[
    "01100000-0000-7000-8000-000000000001", // activity.create
    "01100000-0000-7000-8000-000000000012", // activity.read
    "01100000-0000-7000-8000-000000000002", // activity.update
    "01100000-0000-7000-8000-000000000003", // activity.delete
    "01100000-0000-7000-8000-000000000013", // activity.export
    "01100000-0000-7000-8000-00000000000c", // member.create
    "01100000-0000-7000-8000-00000000001b", // member.read
    "01100000-0000-7000-8000-00000000001c", // member.update
    "01100000-0000-7000-8000-00000000001d", // member.delete
    "01100000-0000-7000-8000-00000000001e", // member.export
    "01100000-0000-7000-8000-00000000001f", // role.create
    "01100000-0000-7000-8000-000000000020", // role.read
    "01100000-0000-7000-8000-000000000021", // role.update
    "01100000-0000-7000-8000-000000000022", // role.delete
    "01100000-0000-7000-8000-000000000023", // role.export
    "01100000-0000-7000-8000-000000000016", // tag.create
    "01100000-0000-7000-8000-000000000017", // tag.read
    "01100000-0000-7000-8000-000000000018", // tag.update
    "01100000-0000-7000-8000-000000000019", // tag.delete
    "01100000-0000-7000-8000-00000000001a", // tag.export
    "01100000-0000-7000-8000-000000000004", // timesheet.create
    "01100000-0000-7000-8000-000000000014", // timesheet.read
    "01100000-0000-7000-8000-000000000005", // timesheet.update
    "01100000-0000-7000-8000-00000000000e", // timesheet.delete
    "01100000-0000-7000-8000-000000000006", // timesheet.export
    "01100000-0000-7000-8000-000000000015", // timesheet.read_all
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // 1. Add is_instance_admin column.
        // SQLite has no native boolean — use INTEGER (0/1) matching the pattern
        // already used for is_verified in m20260512_000001_add_email_verification.
        let add_col_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            "ALTER TABLE projections__users \
             ADD COLUMN is_instance_admin INTEGER NOT NULL DEFAULT 0"
        } else {
            "ALTER TABLE projections__users \
             ADD COLUMN is_instance_admin BOOLEAN NOT NULL DEFAULT FALSE"
        };
        conn.execute_unprepared(add_col_sql).await?;

        // 2. Partial unique index — at most one row may have is_instance_admin = true/1.
        // SQLite uses 1, PostgreSQL uses TRUE; the expression evaluates identically.
        let idx_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            "CREATE UNIQUE INDEX idx_one_instance_admin \
             ON projections__users (is_instance_admin) \
             WHERE is_instance_admin = 1"
        } else {
            "CREATE UNIQUE INDEX idx_one_instance_admin \
             ON projections__users (is_instance_admin) \
             WHERE is_instance_admin = TRUE"
        };
        conn.execute_unprepared(idx_sql).await?;

        // 3. Remove admin.bypass permission and all grants referencing it.
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_role_permissions \
             WHERE permission_id = '{ADMIN_BYPASS_ID}'"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_user_permissions \
             WHERE permission_id = '{ADMIN_BYPASS_ID}'"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "DELETE FROM permissions WHERE id = '{ADMIN_BYPASS_ID}'"
        ))
        .await?;

        // 4a. Rename every remaining "admin" role to "workspace_admin".
        //     (workspaces that already have a "workspace_admin" role are handled by
        //      step 4b below — the rename is skipped for those.)
        conn.execute_unprepared(
            "UPDATE projections__workspace_roles \
             SET name = 'workspace_admin' \
             WHERE name = 'admin' \
               AND workspace_id NOT IN ( \
                 SELECT workspace_id FROM projections__workspace_roles \
                 WHERE name = 'workspace_admin' \
               )",
        )
        .await?;

        // 4b. For any workspace that had BOTH "admin" and "workspace_admin" roles,
        //     the "admin" role still exists. Copy its user assignments to workspace_admin,
        //     then delete it.
        let copy_user_roles_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            "INSERT OR IGNORE INTO projections__workspace_user_roles \
                 (workspace_id, user_id, workspace_role_id) \
             SELECT wur.workspace_id, wur.user_id, wa.id \
             FROM projections__workspace_user_roles wur \
             JOIN projections__workspace_roles ar \
               ON wur.workspace_role_id = ar.id AND ar.name = 'admin' \
             JOIN projections__workspace_roles wa \
               ON wa.workspace_id = wur.workspace_id AND wa.name = 'workspace_admin'"
        } else {
            "INSERT INTO projections__workspace_user_roles \
                 (workspace_id, user_id, workspace_role_id) \
             SELECT wur.workspace_id, wur.user_id, wa.id \
             FROM projections__workspace_user_roles wur \
             JOIN projections__workspace_roles ar \
               ON wur.workspace_role_id = ar.id AND ar.name = 'admin' \
             JOIN projections__workspace_roles wa \
               ON wa.workspace_id = wur.workspace_id AND wa.name = 'workspace_admin' \
             ON CONFLICT DO NOTHING"
        };
        conn.execute_unprepared(copy_user_roles_sql).await?;

        conn.execute_unprepared(
            "DELETE FROM projections__workspace_user_roles \
             WHERE workspace_role_id IN ( \
               SELECT id FROM projections__workspace_roles WHERE name = 'admin' \
             )",
        )
        .await?;

        conn.execute_unprepared(
            "DELETE FROM projections__workspace_role_permissions \
             WHERE workspace_role_id IN ( \
               SELECT id FROM projections__workspace_roles WHERE name = 'admin' \
             )",
        )
        .await?;

        conn.execute_unprepared("DELETE FROM projections__workspace_roles WHERE name = 'admin'")
            .await?;

        // 4c. Grant all 26 workspace permissions to every "workspace_admin" role
        //     (covers both freshly renamed roles and pre-existing ones).
        let perm_union = WORKSPACE_PERM_IDS
            .iter()
            .map(|id| format!("SELECT '{id}' AS pid"))
            .collect::<Vec<_>>()
            .join(" UNION ALL ");

        let grant_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            format!(
                "INSERT OR IGNORE INTO projections__workspace_role_permissions \
                     (workspace_role_id, permission_id) \
                 SELECT r.id, p.pid \
                 FROM projections__workspace_roles r \
                 CROSS JOIN ({perm_union}) AS p \
                 WHERE r.name = 'workspace_admin' \
                   AND EXISTS (SELECT 1 FROM permissions WHERE id = p.pid)"
            )
        } else {
            format!(
                "INSERT INTO projections__workspace_role_permissions \
                     (workspace_role_id, permission_id) \
                 SELECT r.id, p.pid \
                 FROM projections__workspace_roles r \
                 CROSS JOIN ({perm_union}) AS p \
                 WHERE r.name = 'workspace_admin' \
                   AND EXISTS (SELECT 1 FROM permissions WHERE id = p.pid) \
                 ON CONFLICT DO NOTHING"
            )
        };
        conn.execute_unprepared(&grant_sql).await?;

        // 5. Set is_instance_admin for the earliest-created workspace_admin user.
        let set_admin_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            "UPDATE projections__users \
             SET is_instance_admin = 1 \
             WHERE id = ( \
               SELECT wur.user_id \
               FROM projections__workspace_user_roles wur \
               JOIN projections__workspace_roles wr ON wur.workspace_role_id = wr.id \
               WHERE wr.name = 'workspace_admin' \
               ORDER BY wur.user_id ASC \
               LIMIT 1 \
             )"
        } else {
            "UPDATE projections__users \
             SET is_instance_admin = TRUE \
             WHERE id = ( \
               SELECT wur.user_id \
               FROM projections__workspace_user_roles wur \
               JOIN projections__workspace_roles wr ON wur.workspace_role_id = wr.id \
               WHERE wr.name = 'workspace_admin' \
               ORDER BY wur.user_id ASC \
               LIMIT 1 \
             )"
        };
        conn.execute_unprepared(set_admin_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // Re-seed admin.bypass permission.
        let sql = if db == sea_orm::DatabaseBackend::Sqlite {
            format!(
                "INSERT OR IGNORE INTO permissions (id, name) \
                 VALUES ('{ADMIN_BYPASS_ID}', 'admin.bypass')"
            )
        } else {
            format!(
                "INSERT INTO permissions (id, name) \
                 VALUES ('{ADMIN_BYPASS_ID}', 'admin.bypass') ON CONFLICT DO NOTHING"
            )
        };
        conn.execute_unprepared(&sql).await?;

        // Drop the partial unique index.
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_one_instance_admin")
            .await?;

        // Drop the column (SQLite < 3.35 doesn't support DROP COLUMN — skip gracefully).
        let _ = conn
            .execute_unprepared("ALTER TABLE projections__users DROP COLUMN is_instance_admin")
            .await;

        Ok(())
    }
}
