use sea_orm_migration::prelude::*;

/// Cleans up stale and incorrect permission data from existing databases.
///
/// Removes permissions that were seeded in `m20260410_000001` but are no longer
/// part of Zeitrak (`customer.*`, `project.*`, `rate.manage`), and removes
/// `activity.create` / `activity.update` from all "standard" roles (standard
/// members should be able to view activities, not manage them).
///
/// The `down()` migration re-inserts the five removed permissions so the
/// rollback is consistent with reverting `m20260515_000001`.
#[derive(DeriveMigrationName)]
pub struct Migration;

const STALE_PERMISSION_IDS: &[&str] = &[
    "01100000-0000-7000-8000-000000000001", // customer.create
    "01100000-0000-7000-8000-000000000002", // customer.update
    "01100000-0000-7000-8000-000000000003", // project.create
    "01100000-0000-7000-8000-000000000004", // project.update
    "01100000-0000-7000-8000-00000000000b", // rate.manage
];

const STALE_PERMISSIONS_WITH_NAMES: &[(&str, &str)] = &[
    ("01100000-0000-7000-8000-000000000001", "customer.create"),
    ("01100000-0000-7000-8000-000000000002", "customer.update"),
    ("01100000-0000-7000-8000-000000000003", "project.create"),
    ("01100000-0000-7000-8000-000000000004", "project.update"),
    ("01100000-0000-7000-8000-00000000000b", "rate.manage"),
];

const ACTIVITY_PERMISSION_IDS: &[&str] = &[
    "01100000-0000-7000-8000-000000000005", // activity.create
    "01100000-0000-7000-8000-000000000006", // activity.update
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        let stale_ids = STALE_PERMISSION_IDS
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ");

        // Remove stale permissions from role permission assignments.
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_role_permissions \
             WHERE permission_id IN ({stale_ids})"
        ))
        .await?;

        // Remove stale permissions from direct user permission assignments.
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_user_permissions \
             WHERE permission_id IN ({stale_ids})"
        ))
        .await?;

        // Remove stale permissions from the permissions table.
        conn.execute_unprepared(&format!(
            "DELETE FROM permissions WHERE id IN ({stale_ids})"
        ))
        .await?;

        // Remove activity.create/update from all "standard" roles.
        let activity_ids = ACTIVITY_PERMISSION_IDS
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ");

        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_role_permissions \
             WHERE workspace_role_id IN ( \
                 SELECT id FROM projections__workspace_roles WHERE name = 'standard' \
             ) \
             AND permission_id IN ({activity_ids})"
        ))
        .await?;

        let _ = db; // suppress unused warning when both branches compile to same code
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        for (id, name) in STALE_PERMISSIONS_WITH_NAMES {
            let sql = if db == sea_orm::DatabaseBackend::Sqlite {
                format!("INSERT OR IGNORE INTO permissions (id, name) VALUES ('{id}', '{name}')")
            } else {
                format!(
                    "INSERT INTO permissions (id, name) VALUES ('{id}', '{name}') \
                     ON CONFLICT (id) DO NOTHING"
                )
            };
            conn.execute_unprepared(&sql).await?;
        }

        Ok(())
    }
}
