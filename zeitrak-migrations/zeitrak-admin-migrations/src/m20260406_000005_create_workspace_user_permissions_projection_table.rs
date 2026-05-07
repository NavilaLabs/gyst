use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projections__workspace_user_permissions")
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("permission_id"))
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col("workspace_id")
                            .col("user_id")
                            .col("permission_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from("projections__workspace_user_permissions", "workspace_id")
                            .to("projections__workspaces", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from("projections__workspace_user_permissions", "user_id")
                            .to("projections__users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from("projections__workspace_user_permissions", "permission_id")
                            .to("permissions", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table("projections__workspace_user_permissions")
                    .to_owned(),
            )
            .await
    }
}
