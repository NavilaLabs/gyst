use eventually::aggregate::repository::{Getter, Saver};
use eventually_projection::{Projector, RawEvent};
use loom_core::tenant::timesheet_tag::{TimesheetTag, TimesheetTagEvent, TimesheetTagId};
use loom_infrastructure_impl::tenant::timesheet_tag::{
    projectors::TimesheetTagProjector, repositories::TimesheetTagRepository,
};
use loom_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

const TAG_ID: &str = "079d0ce8-facb-7c90-b9d7-287ae4f17c91";
const TS_ID: &str = "079d0ce8-facb-7c90-b9d7-287ae4f17c92";

fn tag_id() -> TimesheetTagId {
    TAG_ID.parse().unwrap()
}

async fn make_repository(fixture: &TestFixture) -> TimesheetTagRepository {
    TimesheetTagRepository::from_pool(fixture.tenant.clone())
        .await
        .expect("TimesheetTagRepository must be created")
}

fn raw_created(id: &TimesheetTagId, name: &str) -> RawEvent {
    let event = TimesheetTagEvent::Created { id: id.clone(), name: name.to_string() };
    RawEvent {
        stream_id: id.to_string(),
        version: 1,
        global_position: 1,
        event_type: "TagCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

fn raw(stream_id: &str, version: i64, event_type: &str, event: &TimesheetTagEvent) -> RawEvent {
    RawEvent {
        stream_id: stream_id.to_string(),
        version,
        global_position: version,
        event_type: event_type.to_string(),
        payload_bytes: serde_json::to_vec(event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

pub mod tests {
    use super::*;

    /// Round-trip a tag through the event store.
    #[tokio::test]
    async fn test_save_and_get_timesheet_tag() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = tag_id();

        let mut root = eventually::aggregate::Root::<TimesheetTag>::record_new(
            TimesheetTagEvent::Created { id: id.clone(), name: "backend".to_string() }.into(),
        )
        .expect("Created event is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.name(), "backend");
        assert_eq!(loaded.version(), 1);
    }

    /// The projector inserts a row into `projections__tags` on `TagCreated`.
    #[tokio::test]
    async fn test_projector_inserts_tag_on_created() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant.clone());
        let id = tag_id();

        projector
            .handle(raw_created(&id, "backend"))
            .await
            .expect("must handle TagCreated");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM projections__timesheet_tags WHERE id = ?")
                .bind(TAG_ID)
                .fetch_one(db.tenant.as_ref())
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    /// The projector updates `name` in `projections__tags` on `TagRenamed`.
    #[tokio::test]
    async fn test_projector_renames_tag() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant.clone());
        let id = tag_id();

        projector.handle(raw_created(&id, "old-name")).await.unwrap();

        let renamed = TimesheetTagEvent::Renamed { name: "new-name".to_string() };
        projector
            .handle(raw(TAG_ID, 2, "TagRenamed", &renamed))
            .await
            .expect("must handle TagRenamed");

        let name: String =
            sqlx::query_scalar("SELECT name FROM projections__timesheet_tags WHERE id = ?")
                .bind(TAG_ID)
                .fetch_one(db.tenant.as_ref())
                .await
                .unwrap();
        assert_eq!(name, "new-name");
    }

    /// After tagging then untagging a timesheet the link row is removed.
    #[tokio::test]
    async fn test_projector_tags_and_untags_timesheet() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant.clone());
        let id = tag_id();
        let ts_id: loom_core::tenant::timesheet::TimesheetId = TS_ID.parse().unwrap();

        projector.handle(raw_created(&id, "backend")).await.unwrap();

        let tagged = TimesheetTagEvent::TimesheetTagged { timesheet_id: ts_id.clone() };
        projector
            .handle(raw(TAG_ID, 2, "TagTimesheetTagged", &tagged))
            .await
            .expect("must handle TagTimesheetTagged");

        let count_after_tag: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projections__timesheet_has_tags \
             WHERE timesheet_id = ? AND timesheet_tag_id = ?",
        )
        .bind(TS_ID)
        .bind(TAG_ID)
        .fetch_one(db.tenant.as_ref())
        .await
        .unwrap();
        assert_eq!(count_after_tag, 1, "link row must exist after tagging");

        let untagged = TimesheetTagEvent::TimesheetUntagged { timesheet_id: ts_id };
        projector
            .handle(raw(TAG_ID, 3, "TagTimesheetUntagged", &untagged))
            .await
            .expect("must handle TagTimesheetUntagged");

        let count_after_untag: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projections__timesheet_has_tags \
             WHERE timesheet_id = ? AND timesheet_tag_id = ?",
        )
        .bind(TS_ID)
        .bind(TAG_ID)
        .fetch_one(db.tenant.as_ref())
        .await
        .unwrap();
        assert_eq!(count_after_untag, 0, "link row must be removed after untagging");
    }

    /// The projector silently ignores unknown event types.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant);

        let result = projector
            .handle(RawEvent {
                stream_id: "stream-1".to_string(),
                version: 1,
                global_position: 1,
                event_type: "UnknownEvent".to_string(),
                payload_bytes: b"{}".to_vec(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await;

        assert!(result.is_ok());
    }

    /// `all()` returns tags that have been projected.
    #[tokio::test]
    async fn test_all_returns_projected_tags() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;
        let id = tag_id();

        projector.handle(raw_created(&id, "release")).await.unwrap();

        // Tags are projected to `projections__timesheet_tags`.
        // Verify the tag row exists via `all()`:
        let all_tags = repo.all().await.expect("all() must succeed");
        let found = all_tags.iter().any(|t| t.get_name() == "release");
        assert!(found, "projected tag must appear in all()");

        // Verify for_timesheet works (returns nothing when no tagging done):
        let for_ts = repo
            .for_timesheet(TS_ID)
            .await
            .expect("for_timesheet must succeed");
        assert!(for_ts.is_empty(), "no tags associated with the timesheet yet");
    }

    /// `for_timesheet` returns the tag after a `TimesheetTagged` event is projected.
    #[tokio::test]
    async fn test_for_timesheet_returns_associated_tags() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetTagProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;
        let id = tag_id();
        let ts_id: loom_core::tenant::timesheet::TimesheetId = TS_ID.parse().unwrap();

        projector.handle(raw_created(&id, "backend")).await.unwrap();

        let tagged = TimesheetTagEvent::TimesheetTagged { timesheet_id: ts_id };
        projector
            .handle(raw(TAG_ID, 2, "TagTimesheetTagged", &tagged))
            .await
            .unwrap();

        let tags = repo
            .for_timesheet(TS_ID)
            .await
            .expect("for_timesheet must succeed");

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].get_name(), "backend");
    }
}
