use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use zeitrak_core::admin::workspace::{Workspace, WorkspaceEvent, WorkspaceId, WorkspaceRepository as _};
use zeitrak_infrastructure_impl::admin::workspace::{
    projectors::WorkspaceProjector, repositories::WorkspaceRepository,
};
use zeitrak_tests::TestFixture;
use sqlx::Row;

fn test_id() -> WorkspaceId {
    "029d0ce8-facb-7c90-b9d7-287ae4f17c91"
        .parse()
        .expect("valid UUID")
}

async fn make_repository(fixture: &TestFixture) -> WorkspaceRepository {
    WorkspaceRepository::from_pool(fixture.admin.clone())
        .await
        .expect("WorkspaceRepository must be created")
}

fn created_event(id: WorkspaceId, name: Option<&str>) -> WorkspaceEvent {
    WorkspaceEvent::Created {
        id,
        name: name.map(str::to_owned),
    }
}

pub mod tests {
    use super::*;

    /// Saving a new workspace and loading it back returns the same state.
    #[tokio::test]
    async fn test_save_and_get_workspace() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = test_id();

        let mut root = eventually::aggregate::Root::<Workspace>::record_new(
            created_event(id.clone(), Some("Acme")).into(),
        )
        .expect("Created event on a new workspace is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.name(), Some("Acme"));
        assert_eq!(loaded.version(), 1);
    }

    /// Applying a second `Created` event to an existing workspace must
    /// return `AlreadyExists` — pure domain logic, no database needed.
    #[test]
    fn test_duplicate_workspace_creation_is_rejected_by_domain() {
        let id = test_id();
        let existing =
            Workspace::apply(None, created_event(id.clone(), Some("Acme"))).expect("first Created");

        let result = Workspace::apply(Some(existing), created_event(id, Some("Other")));
        assert!(
            result.is_err(),
            "second Created on an existing workspace must fail"
        );
    }

    /// The projector must insert a row into the projection table when it
    /// receives a `WorkspaceCreated` event.
    #[tokio::test]
    async fn test_projector_inserts_row_on_workspace_created() {
        let db = TestFixture::setup().await;
        let mut projector = WorkspaceProjector::new(db.admin.clone());

        let id = test_id();
        let event = created_event(id.clone(), Some("Acme"));
        let payload_bytes = serde_json::to_vec(&event).expect("serialization must succeed");

        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 1,
                global_position: 1,
                event_type: "WorkspaceCreated".to_string(),
                payload_bytes,
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("projector must handle WorkspaceCreated");

        let rows = sqlx::query("SELECT name FROM projections__workspaces")
            .fetch_all(db.admin.as_ref())
            .await
            .expect("query must succeed");

        let found = rows.iter().any(|r| {
            let name: Option<String> = r.try_get("name").ok();
            name.as_deref() == Some("Acme")
        });
        assert!(found, "projection table should contain a row for Acme");
    }

    /// The projector must silently ignore event types it does not handle.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = WorkspaceProjector::new(db.admin);

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

        assert!(
            result.is_ok(),
            "unknown event type must not produce an error"
        );
    }

    /// After projection, `WorkspaceRepository::find_workspace_for_user` returns
    /// `None` when no role assignment has been projected for that user.
    #[tokio::test]
    async fn test_find_workspace_for_user_returns_none_when_no_role_assigned() {
        let db = TestFixture::setup().await;
        let mut projector = WorkspaceProjector::new(db.admin.clone());
        let repo = make_repository(&db).await;

        let id = test_id();
        let event = created_event(id.clone(), Some("Acme"));
        let payload_bytes = serde_json::to_vec(&event).expect("serialization must succeed");

        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 1,
                global_position: 1,
                event_type: "WorkspaceCreated".to_string(),
                payload_bytes,
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("projector must handle WorkspaceCreated");

        let result = repo
            .find_workspace_for_user("no-such-user")
            .await
            .expect("query must succeed");

        assert!(
            result.is_none(),
            "no workspace should be found for a user with no role assignment"
        );
    }
}
