use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use loom_core::admin::{
    permission::PermissionId,
    workspace::{WorkspaceEvent, WorkspaceId},
    workspace_role::{WorkspaceRole, WorkspaceRoleEvent, WorkspaceRoleId},
};
use loom_infrastructure_impl::admin::{
    workspace::projectors::WorkspaceProjector,
    workspace_role::{projectors::WorkspaceRoleProjector, repositories::WorkspaceRoleRepository},
};
use loom_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

fn role_id() -> WorkspaceRoleId {
    "049d0ce8-facb-7c90-b9d7-287ae4f17c91"
        .parse()
        .expect("valid UUID")
}
fn workspace_id() -> WorkspaceId {
    "049d0ce8-facb-7c90-b9d7-287ae4f17c92"
        .parse()
        .expect("valid UUID")
}
/// Use a seeded permission ID from the admin migrations seed.
fn perm_id() -> PermissionId {
    "01100000-0000-7000-8000-000000000001"
        .parse()
        .expect("valid seeded permission UUID")
}

async fn make_repository(fixture: &TestFixture) -> WorkspaceRoleRepository {
    WorkspaceRoleRepository::from_pool(fixture.admin.clone())
        .await
        .expect("WorkspaceRoleRepository must be created")
}

fn raw_created(rid: &WorkspaceRoleId, wid: &WorkspaceId) -> RawEvent {
    let event = WorkspaceRoleEvent::Created {
        id: rid.clone(),
        workspace_id: wid.clone(),
        name: Some("admin".to_string()),
    };
    RawEvent {
        stream_id: rid.to_string(),
        version: 1,
        global_position: 1,
        event_type: "WorkspaceRoleCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

/// Projects a workspace row into `projections__workspaces` so FK constraints
/// on `projections__workspace_roles.workspace_id` are satisfied.
async fn project_workspace(fixture: &TestFixture, wid: &WorkspaceId) {
    let event = WorkspaceEvent::Created {
        id: wid.clone(),
        name: Some("Test Workspace".to_string()),
    };
    let raw = RawEvent {
        stream_id: wid.to_string(),
        version: 1,
        global_position: 1,
        event_type: "WorkspaceCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    };
    WorkspaceProjector::new(fixture.admin.clone())
        .handle(raw)
        .await
        .expect("workspace projection must succeed");
}

// ── tests ─────────────────────────────────────────────────────────────────────

pub mod tests {
    use loom_infrastructure::repository::ReadRepository;

    use super::*;

    /// Round-trip a workspace role through the event store.
    #[tokio::test]
    async fn test_save_and_get_workspace_role() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let rid = role_id();
        let wid = workspace_id();

        let mut root = eventually::aggregate::Root::<WorkspaceRole>::record_new(
            WorkspaceRoleEvent::Created {
                id: rid.clone(),
                workspace_id: wid.clone(),
                name: Some("admin".to_string()),
            }
            .into(),
        )
        .expect("Created event is always valid for a new aggregate");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&rid).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &rid);
        assert_eq!(loaded.workspace_id(), &wid);
        assert_eq!(loaded.name(), Some("admin"));
        assert_eq!(loaded.version(), 1);
    }

    /// Duplicate creation is rejected by the domain.
    #[test]
    fn test_duplicate_role_creation_rejected_by_domain() {
        let rid = role_id();
        let wid = workspace_id();
        let existing = WorkspaceRole::apply(
            None,
            WorkspaceRoleEvent::Created {
                id: rid.clone(),
                workspace_id: wid.clone(),
                name: None,
            },
        )
        .unwrap();
        let result = WorkspaceRole::apply(
            Some(existing),
            WorkspaceRoleEvent::Created {
                id: rid,
                workspace_id: wid,
                name: None,
            },
        );
        assert!(result.is_err());
    }

    /// The projector inserts a row into `projections__workspace_roles`.
    #[tokio::test]
    async fn test_projector_inserts_row_on_role_created() {
        let db = TestFixture::setup().await;
        let rid = role_id();
        let wid = workspace_id();

        project_workspace(&db, &wid).await;

        let mut projector = WorkspaceRoleProjector::new(db.admin.clone());
        projector
            .handle(raw_created(&rid, &wid))
            .await
            .expect("projector must handle WorkspaceRoleCreated");

        let repo = make_repository(&db).await;
        let view = repo.find_one(rid.0).await.expect("query must succeed");
        assert!(view.is_some(), "projected role must be findable");
        let v = view.unwrap();
        assert_eq!(v.name(), Some("admin"));
        assert_eq!(v.workspace_id(), &wid);
    }

    /// After `WorkspaceRolePermissionGranted` the link row is created.
    #[tokio::test]
    async fn test_projector_handles_permission_granted() {
        let db = TestFixture::setup().await;
        let rid = role_id();
        let wid = workspace_id();
        let pid = perm_id();

        project_workspace(&db, &wid).await;

        let mut projector = WorkspaceRoleProjector::new(db.admin.clone());
        projector.handle(raw_created(&rid, &wid)).await.unwrap();

        let granted = WorkspaceRoleEvent::PermissionGranted {
            permission_id: pid.clone(),
        };
        projector
            .handle(RawEvent {
                stream_id: rid.to_string(),
                version: 2,
                global_position: 2,
                event_type: "WorkspaceRolePermissionGranted".to_string(),
                payload_bytes: serde_json::to_vec(&granted).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("must handle WorkspaceRolePermissionGranted");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projections__workspace_role_permissions \
             WHERE workspace_role_id = ? AND permission_id = ?",
        )
        .bind(rid.to_string())
        .bind(pid.to_string())
        .fetch_one(db.admin.as_ref())
        .await
        .unwrap();
        assert_eq!(count, 1, "permission link row must exist after grant");
    }

    /// After grant then revoke the link row is removed.
    #[tokio::test]
    async fn test_projector_handles_permission_revoked() {
        let db = TestFixture::setup().await;
        let rid = role_id();
        let wid = workspace_id();
        let pid = perm_id();

        project_workspace(&db, &wid).await;

        let mut projector = WorkspaceRoleProjector::new(db.admin.clone());
        projector.handle(raw_created(&rid, &wid)).await.unwrap();

        let granted = WorkspaceRoleEvent::PermissionGranted {
            permission_id: pid.clone(),
        };
        projector
            .handle(RawEvent {
                stream_id: rid.to_string(),
                version: 2,
                global_position: 2,
                event_type: "WorkspaceRolePermissionGranted".to_string(),
                payload_bytes: serde_json::to_vec(&granted).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .unwrap();

        let revoked = WorkspaceRoleEvent::PermissionRevoked {
            permission_id: pid.clone(),
        };
        projector
            .handle(RawEvent {
                stream_id: rid.to_string(),
                version: 3,
                global_position: 3,
                event_type: "WorkspaceRolePermissionRevoked".to_string(),
                payload_bytes: serde_json::to_vec(&revoked).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("must handle WorkspaceRolePermissionRevoked");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projections__workspace_role_permissions \
             WHERE workspace_role_id = ? AND permission_id = ?",
        )
        .bind(rid.to_string())
        .bind(pid.to_string())
        .fetch_one(db.admin.as_ref())
        .await
        .unwrap();
        assert_eq!(count, 0, "permission link row must be removed after revoke");
    }

    /// The projector silently ignores unknown event types.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = WorkspaceRoleProjector::new(db.admin);

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
}
