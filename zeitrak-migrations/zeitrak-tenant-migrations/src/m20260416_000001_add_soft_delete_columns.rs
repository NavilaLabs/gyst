use sea_orm_migration::{prelude::*, schema::timestamp_with_time_zone_null};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("projections__timesheets")
                    .add_column(timestamp_with_time_zone_null("cancelled_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("projections__activities")
                    .add_column(timestamp_with_time_zone_null("deleted_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("projections__timesheets")
                    .drop_column("cancelled_at")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("projections__activities")
                    .drop_column("deleted_at")
                    .to_owned(),
            )
            .await
    }
}
