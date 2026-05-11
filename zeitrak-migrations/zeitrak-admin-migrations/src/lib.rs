pub use sea_orm_migration::prelude::*;
pub use zeitrak_shared_migrations::Error;

mod m20251214_102200_create_event_streams_table;
mod m20251214_102201_create_events_table;
mod m20251215_183831_create_snapshots_table;
mod m20260325_145345_add_global_position;
mod m20260325_185315_create_users_projection_table;
mod m20260406_000001_create_permissions_table;
mod m20260406_000002_create_workspaces_projection_table;
mod m20260406_000003_create_workspace_roles_projection_table;
mod m20260406_000004_create_workspace_user_roles_projection_table;
mod m20260406_000005_create_workspace_user_permissions_projection_table;
mod m20260406_000006_create_workspace_role_permissions_projection_table;
mod m20260410_000001_seed_permissions;
mod m20260410_000002_add_user_settings;
mod m20260410_000003_add_workspace_settings;
mod m20260410_000004_fix_date_format_strings;
mod m20260410_000005_add_aggregate_type_to_event_streams;
mod m20260509_000001_create_invitations_projection_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251214_102200_create_event_streams_table::Migration),
            Box::new(m20251214_102201_create_events_table::Migration),
            Box::new(m20251215_183831_create_snapshots_table::Migration),
            Box::new(m20260325_145345_add_global_position::Migration),
            Box::new(m20260325_185315_create_users_projection_table::Migration),
            Box::new(m20260406_000001_create_permissions_table::Migration),
            Box::new(m20260406_000002_create_workspaces_projection_table::Migration),
            Box::new(m20260406_000003_create_workspace_roles_projection_table::Migration),
            Box::new(m20260406_000004_create_workspace_user_roles_projection_table::Migration),
            Box::new(
                m20260406_000005_create_workspace_user_permissions_projection_table::Migration,
            ),
            Box::new(
                m20260406_000006_create_workspace_role_permissions_projection_table::Migration,
            ),
            Box::new(m20260410_000001_seed_permissions::Migration),
            Box::new(m20260410_000002_add_user_settings::Migration),
            Box::new(m20260410_000003_add_workspace_settings::Migration),
            Box::new(m20260410_000004_fix_date_format_strings::Migration),
            Box::new(m20260410_000005_add_aggregate_type_to_event_streams::Migration),
            Box::new(m20260509_000001_create_invitations_projection_table::Migration),
        ]
    }
}
