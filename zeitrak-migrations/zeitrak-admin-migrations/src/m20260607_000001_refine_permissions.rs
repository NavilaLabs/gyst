use sea_orm_migration::prelude::*;

/// Replaces coarse-grained permissions with a full CRUD+export model per aggregate.
///
/// Removed (replaced by fine-grained equivalents):
/// - `tag.manage`   → `tag.create/read/update/delete/export`
/// - `member.manage` → `member.read/update/delete/export`
/// - `role.manage`  → `role.create/read/update/delete/export`
///
/// Renamed (same UUID, new name):
/// - `timesheet.cancel` → `timesheet.delete`
/// - `member.invite`    → `member.create`
///
/// New permissions added:
/// - `activity.read`, `activity.export`
/// - `timesheet.read`, `timesheet.read_all`
/// - `tag.create/read/update/delete/export`
/// - `member.read/update/delete/export`
/// - `role.create/read/update/delete/export`
///
/// All role-permission grants for removed permissions are cascade-deleted before
/// the permissions themselves are removed.
#[derive(DeriveMigrationName)]
pub struct Migration;

const RENAME: &[(&str, &str)] = &[
    ("01100000-0000-7000-8000-00000000000e", "timesheet.delete"),
    ("01100000-0000-7000-8000-00000000000c", "member.create"),
];

const REMOVE_IDS: &[&str] = &[
    "01100000-0000-7000-8000-00000000000a", // tag.manage
    "01100000-0000-7000-8000-00000000000f", // member.manage
    "01100000-0000-7000-8000-000000000010", // role.manage
];

const ADD: &[(&str, &str)] = &[
    ("01100000-0000-7000-8000-000000000012", "activity.read"),
    ("01100000-0000-7000-8000-000000000013", "activity.export"),
    ("01100000-0000-7000-8000-000000000014", "timesheet.read"),
    ("01100000-0000-7000-8000-000000000015", "timesheet.read_all"),
    ("01100000-0000-7000-8000-000000000016", "tag.create"),
    ("01100000-0000-7000-8000-000000000017", "tag.read"),
    ("01100000-0000-7000-8000-000000000018", "tag.update"),
    ("01100000-0000-7000-8000-000000000019", "tag.delete"),
    ("01100000-0000-7000-8000-00000000001a", "tag.export"),
    ("01100000-0000-7000-8000-00000000001b", "member.read"),
    ("01100000-0000-7000-8000-00000000001c", "member.update"),
    ("01100000-0000-7000-8000-00000000001d", "member.delete"),
    ("01100000-0000-7000-8000-00000000001e", "member.export"),
    ("01100000-0000-7000-8000-00000000001f", "role.create"),
    ("01100000-0000-7000-8000-000000000020", "role.read"),
    ("01100000-0000-7000-8000-000000000021", "role.update"),
    ("01100000-0000-7000-8000-000000000022", "role.delete"),
    ("01100000-0000-7000-8000-000000000023", "role.export"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // Rename in-place (same UUIDs, new names).
        for (id, new_name) in RENAME {
            conn.execute_unprepared(&format!(
                "UPDATE permissions SET name = '{new_name}' WHERE id = '{id}'"
            ))
            .await?;
        }

        // Remove coarse permissions and their role/user grants.
        let remove_list = REMOVE_IDS
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ");

        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_role_permissions WHERE permission_id IN ({remove_list})"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_user_permissions WHERE permission_id IN ({remove_list})"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "DELETE FROM permissions WHERE id IN ({remove_list})"
        ))
        .await?;

        // Seed new fine-grained permissions.
        for (id, name) in ADD {
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // Reverse renames.
        conn.execute_unprepared(
            "UPDATE permissions SET name = 'timesheet.cancel' \
             WHERE id = '01100000-0000-7000-8000-00000000000e'",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE permissions SET name = 'member.invite' \
             WHERE id = '01100000-0000-7000-8000-00000000000c'",
        )
        .await?;

        // Remove new fine-grained permissions.
        let add_ids = ADD
            .iter()
            .map(|(id, _)| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ");
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_role_permissions WHERE permission_id IN ({add_ids})"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "DELETE FROM projections__workspace_user_permissions WHERE permission_id IN ({add_ids})"
        ))
        .await?;
        conn.execute_unprepared(&format!("DELETE FROM permissions WHERE id IN ({add_ids})"))
            .await?;

        // Re-insert removed coarse permissions.
        let coarse = &[
            ("01100000-0000-7000-8000-00000000000a", "tag.manage"),
            ("01100000-0000-7000-8000-00000000000f", "member.manage"),
            ("01100000-0000-7000-8000-000000000010", "role.manage"),
        ];
        for (id, name) in coarse {
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
