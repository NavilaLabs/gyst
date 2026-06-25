use sea_orm_migration::{prelude::*, schema::string_null};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projections__workspace_roles")
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("workspace_id")).string().not_null())
                    .col(string_null("name"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("projections__workspace_roles", "workspace_id")
                            .to("projections__workspaces", "id")
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
                    .table("projections__workspace_roles")
                    .to_owned(),
            )
            .await
    }
}
