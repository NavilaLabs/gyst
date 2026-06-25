use sea_orm_migration::{
    prelude::*,
    schema::{
        integer_null, string, string_null, timestamp_with_time_zone,
        timestamp_with_time_zone_null,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projections__timesheets")
                    .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("user_id")).string().not_null())
                    .col(ColumnDef::new(Alias::new("activity_id")).string())
                    .col(timestamp_with_time_zone("start_time"))
                    .col(timestamp_with_time_zone_null("end_time"))
                    .col(integer_null("duration"))
                    .col(string_null("description"))
                    .col(string("timezone").default("Europe/Berlin"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    // No FK on user_id — users live in the admin database.
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_timesheets_activity_id")
                            .from(
                                TableRef::Table("projections__timesheets".into(), None),
                                "activity_id",
                            )
                            .to(
                                TableRef::Table("projections__activities".into(), None),
                                "id",
                            )
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table("projections__timesheets")
                    .name("idx_timesheets_user_start")
                    .col("user_id")
                    .col("start_time")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("projections__timesheets").to_owned())
            .await
    }
}
