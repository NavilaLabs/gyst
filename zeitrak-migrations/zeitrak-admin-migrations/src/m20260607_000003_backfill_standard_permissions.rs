use sea_orm_migration::prelude::*;

/// Backfills `activity.read`, `tag.read`, and `timesheet.read` onto every
/// existing "standard" workspace role.
///
/// These three permissions were introduced in `m20260607_000001_refine_permissions`
/// and are seeded by `create_workspace_for_user` for new workspaces, but the
/// refine-permissions migration only added the permission rows — it never
/// granted them to already-existing standard roles.
#[derive(DeriveMigrationName)]
pub struct Migration;

const ACTIVITY_READ_ID: &str = "01100000-0000-7000-8000-000000000012";
const TAG_READ_ID: &str = "01100000-0000-7000-8000-000000000017";
const TIMESHEET_READ_ID: &str = "01100000-0000-7000-8000-000000000014";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        for perm_id in [ACTIVITY_READ_ID, TAG_READ_ID, TIMESHEET_READ_ID] {
            let sql = if db == sea_orm::DatabaseBackend::Sqlite {
                format!(
                    "INSERT OR IGNORE INTO projections__workspace_role_permissions \
                         (workspace_role_id, permission_id) \
                     SELECT r.id, '{perm_id}' \
                     FROM projections__workspace_roles r \
                     WHERE r.name = 'standard' \
                       AND EXISTS (SELECT 1 FROM permissions WHERE id = '{perm_id}')"
                )
            } else {
                format!(
                    "INSERT INTO projections__workspace_role_permissions \
                         (workspace_role_id, permission_id) \
                     SELECT r.id, '{perm_id}' \
                     FROM projections__workspace_roles r \
                     WHERE r.name = 'standard' \
                       AND EXISTS (SELECT 1 FROM permissions WHERE id = '{perm_id}') \
                     ON CONFLICT DO NOTHING"
                )
            };
            conn.execute_unprepared(&sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        for perm_id in [ACTIVITY_READ_ID, TAG_READ_ID, TIMESHEET_READ_ID] {
            conn.execute_unprepared(&format!(
                "DELETE FROM projections__workspace_role_permissions \
                 WHERE permission_id = '{perm_id}' \
                   AND workspace_role_id IN ( \
                     SELECT id FROM projections__workspace_roles WHERE name = 'standard' \
                   )"
            ))
            .await?;
        }

        Ok(())
    }
}
