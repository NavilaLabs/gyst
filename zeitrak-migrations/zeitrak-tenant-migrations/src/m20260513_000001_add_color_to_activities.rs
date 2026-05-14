use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("projections__activities")
                    .add_column(
                        ColumnDef::new("color")
                            .string_len(7)
                            .not_null()
                            .default("#6c6c76"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("projections__activities")
                    .drop_column("color")
                    .to_owned(),
            )
            .await
    }
}
