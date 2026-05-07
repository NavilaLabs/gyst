use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        match manager.get_database_backend() {
            sea_orm::DatabaseBackend::Postgres => {
                db.execute_unprepared("CREATE SEQUENCE IF NOT EXISTS events_global_position_seq")
                    .await?;
                db.execute_unprepared(
                    "ALTER TABLE events \
                        ADD COLUMN global_position BIGINT NOT NULL \
                        DEFAULT nextval('events_global_position_seq')",
                )
                .await?;
                db.execute_unprepared(
                    "CREATE INDEX IF NOT EXISTS events_global_position_idx \
                        ON events (global_position)",
                )
                .await?;
            }
            sea_orm::DatabaseBackend::Sqlite => {
                db.execute_unprepared(
                    "ALTER TABLE events ADD COLUMN global_position INTEGER NOT NULL DEFAULT 0",
                )
                .await?;
                db.execute_unprepared("UPDATE events SET global_position = rowid")
                    .await?;
                db.execute_unprepared("DROP TRIGGER IF EXISTS events_assign_global_position")
                    .await?;
                db.execute_unprepared(
                    "CREATE TRIGGER events_assign_global_position \
                    AFTER INSERT ON events \
                    FOR EACH ROW \
                    WHEN NEW.global_position = 0 \
                    BEGIN \
                        UPDATE events \
                        SET global_position = ( \
                            SELECT COALESCE(MAX(global_position), 0) + 1 \
                            FROM events \
                            WHERE global_position != 0 \
                        ) \
                        WHERE rowid = NEW.rowid; \
                    END",
                )
                .await?;
                db.execute_unprepared(
                    "CREATE INDEX IF NOT EXISTS events_global_position_idx \
                        ON events (global_position)",
                )
                .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        match manager.get_database_backend() {
            sea_orm::DatabaseBackend::Postgres => {
                db.execute_unprepared("DROP INDEX IF EXISTS events_global_position_idx")
                    .await?;
                db.execute_unprepared("ALTER TABLE events DROP COLUMN IF EXISTS global_position")
                    .await?;
                db.execute_unprepared("DROP SEQUENCE IF EXISTS events_global_position_seq")
                    .await?;
            }
            sea_orm::DatabaseBackend::Sqlite => {
                db.execute_unprepared("DROP TRIGGER IF EXISTS events_assign_global_position")
                    .await?;
                db.execute_unprepared("DROP INDEX IF EXISTS events_global_position_idx")
                    .await?;
                db.execute_unprepared("CREATE TABLE events_backup AS SELECT * FROM events")
                    .await?;
                db.execute_unprepared("DROP TABLE events").await?;
                db.execute_unprepared("ALTER TABLE events_backup RENAME TO events")
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
