use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use loom_core::tenant::timesheet::{Timesheet, TimesheetEvent, TimesheetId};
use loom_infrastructure_impl::tenant::timesheet::{
    projectors::TimesheetProjector, repositories::TimesheetRepository,
};
use loom_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

const TS_ID: &str = "069d0ce8-facb-7c90-b9d7-287ae4f17c91";
const USER_ID: &str = "069d0ce8-facb-7c90-b9d7-287ae4f17c92";

fn ts_id() -> TimesheetId {
    TS_ID.parse().unwrap()
}

async fn make_repository(fixture: &TestFixture) -> TimesheetRepository {
    TimesheetRepository::from_pool(fixture.tenant.clone())
        .await
        .expect("TimesheetRepository must be created")
}

fn started_event() -> TimesheetEvent {
    TimesheetEvent::Started {
        id: TS_ID.parse().unwrap(),
        user_id: USER_ID.parse().unwrap(),
        activity_id: None,
        start_time: "2024-01-01T09:00:00Z".to_string(),
        timezone: "UTC".to_string(),
    }
}

fn raw(stream_id: &str, version: i64, event_type: &str, event: &TimesheetEvent) -> RawEvent {
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

    /// Round-trip a timesheet through the event store.
    #[tokio::test]
    async fn test_save_and_get_timesheet() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = ts_id();

        let mut root = eventually::aggregate::Root::<Timesheet>::record_new(
            started_event().into(),
        )
        .expect("Started event is always valid for a new aggregate");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.start_time(), "2024-01-01T09:00:00Z");
        assert_eq!(loaded.version(), 1);
    }

    /// Duplicate creation is rejected by the domain.
    #[test]
    fn test_duplicate_timesheet_start_rejected_by_domain() {
        let existing = Timesheet::apply(None, started_event()).unwrap();
        let result = Timesheet::apply(Some(existing), started_event());
        assert!(result.is_err());
    }

    /// The projector inserts a row on `TimesheetStarted`.
    #[tokio::test]
    async fn test_projector_inserts_row_on_started() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());

        let ev = started_event();
        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &ev))
            .await
            .expect("must handle TimesheetStarted");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projections__timesheets WHERE id = ?",
        )
        .bind(TS_ID)
        .fetch_one(db.tenant.as_ref())
        .await
        .unwrap();
        assert_eq!(count, 1);
    }

    /// The projector sets `end_time` and `duration` on `TimesheetStopped`.
    #[tokio::test]
    async fn test_projector_updates_on_stopped() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());

        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &started_event()))
            .await
            .unwrap();

        let stopped = TimesheetEvent::Stopped {
            end_time: "2024-01-01T10:00:00Z".to_string(),
            duration: 3600,
        };
        projector
            .handle(raw(TS_ID, 2, "TimesheetStopped", &stopped))
            .await
            .expect("must handle TimesheetStopped");

        let end_time: Option<String> = sqlx::query_scalar(
            "SELECT end_time FROM projections__timesheets WHERE id = ?",
        )
        .bind(TS_ID)
        .fetch_one(db.tenant.as_ref())
        .await
        .unwrap();
        assert_eq!(end_time.as_deref(), Some("2024-01-01T10:00:00Z"));
    }

    /// The projector updates `description` on `TimesheetUpdated`.
    #[tokio::test]
    async fn test_projector_updates_description() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());

        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &started_event()))
            .await
            .unwrap();

        let updated = TimesheetEvent::Updated { description: Some("pair session".to_string()) };
        projector
            .handle(raw(TS_ID, 2, "TimesheetUpdated", &updated))
            .await
            .expect("must handle TimesheetUpdated");

        let desc: Option<String> = sqlx::query_scalar(
            "SELECT description FROM projections__timesheets WHERE id = ?",
        )
        .bind(TS_ID)
        .fetch_one(db.tenant.as_ref())
        .await
        .unwrap();
        assert_eq!(desc.as_deref(), Some("pair session"));
    }

    /// The projector silently ignores unknown event types.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant);

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

    /// `running_for_user` returns `None` when the user has no running timesheet.
    #[tokio::test]
    async fn test_running_for_user_returns_none_when_no_running_timesheet() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;

        let result = repo
            .running_for_user(USER_ID)
            .await
            .expect("query must succeed");
        assert!(result.is_none());
    }

    /// `running_for_user` returns the timesheet after it has been projected as started.
    #[tokio::test]
    async fn test_running_for_user_returns_started_timesheet() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;

        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &started_event()))
            .await
            .unwrap();

        let result = repo
            .running_for_user(USER_ID)
            .await
            .expect("query must succeed");
        assert!(result.is_some(), "running timesheet must be returned");
        assert_eq!(result.unwrap().get_start_time(), "2024-01-01T09:00:00Z");
    }

    /// `running_for_user` returns `None` once the timesheet has been stopped.
    #[tokio::test]
    async fn test_running_for_user_returns_none_after_stopped() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;

        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &started_event()))
            .await
            .unwrap();
        let stopped = TimesheetEvent::Stopped {
            end_time: "2024-01-01T10:00:00Z".to_string(),
            duration: 3600,
        };
        projector
            .handle(raw(TS_ID, 2, "TimesheetStopped", &stopped))
            .await
            .unwrap();

        let result = repo
            .running_for_user(USER_ID)
            .await
            .expect("query must succeed");
        assert!(result.is_none(), "no running timesheet after stop");
    }

    /// `recent_for_user` returns the projected timesheet.
    #[tokio::test]
    async fn test_recent_for_user_returns_projected_timesheets() {
        let db = TestFixture::setup().await;
        let mut projector = TimesheetProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;

        projector
            .handle(raw(TS_ID, 1, "TimesheetStarted", &started_event()))
            .await
            .unwrap();

        let recent = repo
            .recent_for_user(USER_ID)
            .await
            .expect("query must succeed");
        assert!(!recent.is_empty(), "must return the started timesheet");
    }
}
