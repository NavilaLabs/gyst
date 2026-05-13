use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use zeitrak_core::admin::permission::{Permission, PermissionEvent, PermissionId};
use zeitrak_core::shared::repositories::ReadRepository;
use zeitrak_infrastructure_impl::admin::permission::{
    projectors::PermissionProjector, repositories::PermissionRepository,
};
use zeitrak_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

fn test_id() -> PermissionId {
    "039d0ce8-facb-7c90-b9d7-287ae4f17c91"
        .parse()
        .expect("valid UUID")
}

async fn make_repository(fixture: &TestFixture) -> PermissionRepository {
    PermissionRepository::from_pool(fixture.admin.clone())
        .await
        .expect("PermissionRepository must be created")
}

fn raw_created(id: &PermissionId, name: &str) -> RawEvent {
    let event = PermissionEvent::Created {
        id: id.clone(),
        name: name.to_string(),
    };
    RawEvent {
        stream_id: id.to_string(),
        version: 1,
        global_position: 1,
        event_type: "PermissionCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

pub mod tests {
    use super::*;

    /// Saving a new permission and loading it back returns the same state.
    #[tokio::test]
    async fn test_save_and_get_permission() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = test_id();

        let mut root = eventually::aggregate::Root::<Permission>::record_new(
            PermissionEvent::Created {
                id: id.clone(),
                name: "can_invite".to_string(),
            }
            .into(),
        )
        .expect("Created event on a new aggregate is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.name(), "can_invite");
        assert_eq!(loaded.version(), 1);
    }

    /// Double-creating the same permission is rejected by domain logic.
    #[test]
    fn test_duplicate_permission_creation_rejected_by_domain() {
        let id = test_id();
        let existing = Permission::apply(
            None,
            PermissionEvent::Created {
                id: id.clone(),
                name: "can_invite".to_string(),
            },
        )
        .expect("first Created is valid");

        let result = Permission::apply(
            Some(existing),
            PermissionEvent::Created {
                id,
                name: "duplicate".to_string(),
            },
        );
        assert!(result.is_err(), "second Created must fail");
    }

    /// The projector inserts a row into `permissions` on `PermissionCreated`.
    #[tokio::test]
    async fn test_projector_inserts_row_on_permission_created() {
        let db = TestFixture::setup().await;
        let mut projector = PermissionProjector::new(db.admin.clone());
        let id = test_id();

        projector
            .handle(raw_created(&id, "can_invite"))
            .await
            .expect("projector must handle PermissionCreated");

        let repo = make_repository(&db).await;
        let view = repo.find(id).await.expect("query must succeed");
        assert!(view.is_some(), "projected permission must be findable");
        assert_eq!(view.unwrap().name(), "can_invite");
    }

    /// The projector is idempotent — sending the same event twice must not error.
    #[tokio::test]
    async fn test_projector_is_idempotent() {
        let db = TestFixture::setup().await;
        let mut projector = PermissionProjector::new(db.admin.clone());
        let id = test_id();

        projector
            .handle(raw_created(&id, "can_invite"))
            .await
            .unwrap();
        let result = projector.handle(raw_created(&id, "can_invite")).await;
        assert!(result.is_ok(), "second identical event must not error");
    }

    /// The projector silently ignores event types it does not handle.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = PermissionProjector::new(db.admin);

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

    /// The migrations seed a set of known permissions; `all()` must return them.
    #[tokio::test]
    async fn test_all_returns_seeded_permissions() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;

        let all = repo.all().await.expect("all() must succeed");
        // The seed migration inserts at least one permission.
        assert!(!all.is_empty(), "seeded permissions must be present");
    }

    /// `count()` matches the number of rows returned by `all()`.
    #[tokio::test]
    async fn test_count_matches_all_len() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;

        let count = repo.count().await.expect("count must succeed");
        let all = repo.all().await.expect("all must succeed");
        assert_eq!(count, all.len() as u64);
    }

    /// `find_one` returns `None` for an ID that was never projected.
    #[tokio::test]
    async fn test_find_one_returns_none_for_unknown_id() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let unknown_id: sqlx::types::Uuid = "ffffffff-ffff-7fff-bfff-ffffffffffff".parse().unwrap();

        let result = repo
            .find(zeitrak_core::shared::AggregateId(unknown_id))
            .await
            .expect("query must succeed");
        assert!(result.is_none());
    }
}
