use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use sqlx::Row;
use zeitrak_core::tenant::activity::{Activity, ActivityEvent, ActivityId};
use zeitrak_infrastructure_impl::tenant::activity::{
    projectors::ActivityProjector, repositories::ActivityRepository,
};
use zeitrak_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

fn test_id() -> ActivityId {
    "059d0ce8-facb-7c90-b9d7-287ae4f17c91"
        .parse()
        .expect("valid UUID")
}

async fn make_repository(fixture: &TestFixture) -> ActivityRepository {
    ActivityRepository::from_pool(fixture.tenant.clone())
        .await
        .expect("ActivityRepository must be created")
}

fn raw_created(id: &ActivityId, name: &str) -> RawEvent {
    let event = ActivityEvent::Created {
        id: id.clone(),
        name: name.to_string(),
        comment: None,
    };
    RawEvent {
        stream_id: id.to_string(),
        version: 1,
        global_position: 1,
        event_type: "ActivityCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

pub mod tests {
    use super::*;

    /// Round-trip an activity through the event store.
    #[tokio::test]
    async fn test_save_and_get_activity() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = test_id();

        let mut root = eventually::aggregate::Root::<Activity>::record_new(
            ActivityEvent::Created {
                id: id.clone(),
                name: "Stand-up".to_string(),
                comment: Some("daily sync".to_string()),
            }
            .into(),
        )
        .expect("Created event is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.name(), "Stand-up");
        assert_eq!(loaded.version(), 1);
    }

    /// Duplicate creation is rejected by the domain.
    #[test]
    fn test_duplicate_activity_creation_rejected_by_domain() {
        let id = test_id();
        let existing = Activity::apply(
            None,
            ActivityEvent::Created {
                id: id.clone(),
                name: "First".to_string(),
                comment: None,
            },
        )
        .unwrap();
        let result = Activity::apply(
            Some(existing),
            ActivityEvent::Created {
                id,
                name: "Second".to_string(),
                comment: None,
            },
        );
        assert!(result.is_err());
    }

    /// The projector inserts a row into `projections__activities` on `ActivityCreated`.
    #[tokio::test]
    async fn test_projector_inserts_row_on_activity_created() {
        let db = TestFixture::setup().await;
        let mut projector = ActivityProjector::new(db.tenant.clone());
        let id = test_id();

        projector
            .handle(raw_created(&id, "Stand-up"))
            .await
            .expect("projector must handle ActivityCreated");

        let rows = sqlx::query("SELECT name FROM projections__activities")
            .fetch_all(db.tenant.as_ref())
            .await
            .unwrap();

        let found = rows
            .iter()
            .any(|r| r.get::<String, _>("name") == "Stand-up");
        assert!(found, "projection table must contain the activity");
    }

    /// The projector updates the row on `ActivityUpdated`.
    #[tokio::test]
    async fn test_projector_updates_row_on_activity_updated() {
        let db = TestFixture::setup().await;
        let mut projector = ActivityProjector::new(db.tenant.clone());
        let id = test_id();

        projector
            .handle(raw_created(&id, "Old Name"))
            .await
            .unwrap();

        let updated = ActivityEvent::Updated {
            name: "New Name".to_string(),
            comment: Some("now with a comment".to_string()),
        };
        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 2,
                global_position: 2,
                event_type: "ActivityUpdated".to_string(),
                payload_bytes: serde_json::to_vec(&updated).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("projector must handle ActivityUpdated");

        let row = sqlx::query("SELECT name FROM projections__activities WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(db.tenant.as_ref())
            .await
            .unwrap();
        assert_eq!(row.get::<String, _>("name"), "New Name");
    }

    /// The projector is idempotent — sending the same event twice must not error.
    #[tokio::test]
    async fn test_projector_is_idempotent() {
        let db = TestFixture::setup().await;
        let mut projector = ActivityProjector::new(db.tenant.clone());
        let id = test_id();

        projector.handle(raw_created(&id, "X")).await.unwrap();
        let result = projector.handle(raw_created(&id, "X")).await;
        assert!(result.is_ok());
    }

    /// The projector silently ignores unknown event types.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = ActivityProjector::new(db.tenant);

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

    /// `ActivityRepository::all()` returns rows that have been projected.
    #[tokio::test]
    async fn test_all_returns_projected_activities() {
        let db = TestFixture::setup().await;
        let mut projector = ActivityProjector::new(db.tenant.clone());
        let repo = make_repository(&db).await;
        let id = test_id();

        projector
            .handle(raw_created(&id, "Sprint-review"))
            .await
            .unwrap();

        let all = repo.all().await.expect("all must succeed");
        let found = all.iter().any(|r| r.name() == "Sprint-review");
        assert!(found, "projected activity must appear in all()");
    }
}
