use eventually::aggregate::repository::{Getter, Saver};
use eventually_projection::{Projector, RawEvent};
use sqlx::Row as _;
use zeitrak_core::admin::{
    invitation::{
        Invitation, InvitationCommand, InvitationCommandTrait, InvitationEvent, InvitationId,
        InvitationRepository as InvitationRepositoryTrait, InvitationStatus,
    },
    user::UserId,
    workspace::WorkspaceId,
    workspace_role::WorkspaceRoleId,
};
use zeitrak_infrastructure_impl::admin::invitation::{
    projectors::InvitationProjector, repositories::InvitationRepository,
};
use zeitrak_tests::TestFixture;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Seeds the projection tables that `projections__invitations` has FK constraints on.
///
/// The invitations projection requires a workspace, a user, and a workspace role
/// to exist as parent rows before an invitation row can be inserted.
async fn seed_fk_parents(pool: &sqlx::AnyPool) {
    sqlx::query("INSERT INTO projections__workspaces (id, name) VALUES ($1, $2)")
        .bind(workspace_id().to_string())
        .bind("Test Workspace")
        .execute(pool)
        .await
        .expect("workspace seed must succeed");

    sqlx::query(
        "INSERT INTO projections__users (id, name, email, password) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id().to_string())
    .bind("Inviter User")
    .bind("inviter@example.com")
    .bind("$2b$12$placeholder")
    .execute(pool)
    .await
    .expect("user seed must succeed");

    sqlx::query(
        "INSERT INTO projections__workspace_roles (id, workspace_id, name) VALUES ($1, $2, $3)",
    )
    .bind(role_id().to_string())
    .bind(workspace_id().to_string())
    .bind("member")
    .execute(pool)
    .await
    .expect("workspace_role seed must succeed");
}

fn invitation_id() -> InvitationId {
    "019d0ce8-facb-7c90-b9d7-000000000010"
        .parse()
        .expect("valid UUID")
}

fn workspace_id() -> WorkspaceId {
    "019d0ce8-facb-7c90-b9d7-000000000020"
        .parse()
        .expect("valid UUID")
}

fn user_id() -> UserId {
    "019d0ce8-facb-7c90-b9d7-000000000030"
        .parse()
        .expect("valid UUID")
}

fn role_id() -> WorkspaceRoleId {
    "019d0ce8-facb-7c90-b9d7-000000000040"
        .parse()
        .expect("valid UUID")
}

async fn make_repository(fixture: &TestFixture) -> InvitationRepository {
    InvitationRepository::from_pool(fixture.admin.clone())
        .await
        .expect("InvitationRepository must be created")
}

fn raw_created_event(id: &InvitationId) -> RawEvent {
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    let event = InvitationEvent::Created {
        id: id.clone(),
        workspace_id: workspace_id(),
        invited_by: user_id(),
        email: "bob@example.com".to_string(),
        workspace_role_id: role_id(),
        token: "test-token-abc".to_string(),
        expires_at,
    };
    RawEvent {
        stream_id: id.to_string(),
        version: 1,
        global_position: 1,
        event_type: "InvitationCreated".to_string(),
        payload_bytes: serde_json::to_vec(&event).unwrap(),
        metadata: serde_json::Value::Null,
        schema_version: 1,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

pub mod tests {
    use super::*;

    /// Round-trip an invitation through the event store: save then get.
    #[tokio::test]
    async fn test_save_and_get_invitation() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = invitation_id();
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        let mut root = eventually::aggregate::Root::<Invitation>::record_new(
            InvitationEvent::Created {
                id: id.clone(),
                workspace_id: workspace_id(),
                invited_by: user_id(),
                email: "alice@example.com".to_string(),
                workspace_role_id: role_id(),
                token: "round-trip-token".to_string(),
                expires_at,
            }
            .into(),
        )
        .expect("Created event on a new aggregate is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.email(), "alice@example.com");
        assert_eq!(loaded.version(), 1);
        assert!(matches!(loaded.status(), InvitationStatus::Pending));
    }

    /// `InvitationCommand::create` produces a pending invitation in the event store.
    #[tokio::test]
    async fn test_command_create_returns_pending_invitation() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let cmd = InvitationCommand::new(repo);
        let id = invitation_id();

        let root = cmd
            .create(
                id.clone(),
                workspace_id(),
                user_id(),
                "carol@example.com".to_string(),
                role_id(),
                7,
            )
            .await
            .expect("create must succeed");

        assert_eq!(root.aggregate_id(), &id);
        assert!(root.is_pending());
        assert!(!root.token().is_empty());
        assert_eq!(root.email(), "carol@example.com");
    }

    /// `InvitationCommand::accept` transitions the invitation to `Accepted`.
    #[tokio::test]
    async fn test_command_accept_transitions_to_accepted() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let cmd = InvitationCommand::new(repo);
        let id = invitation_id();

        let _created = cmd
            .create(
                id.clone(),
                workspace_id(),
                user_id(),
                "dave@example.com".to_string(),
                role_id(),
                7,
            )
            .await
            .expect("create must succeed");

        let accepted_by: UserId = "019d0ce8-facb-7c90-b9d7-000000000050"
            .parse()
            .expect("valid UUID");
        cmd.accept(id.clone(), accepted_by)
            .await
            .expect("accept must succeed");

        let repo2 = make_repository(&db).await;
        let loaded = repo2.get(&id).await.expect("get must succeed");
        assert!(
            matches!(loaded.status(), InvitationStatus::Accepted),
            "invitation must be Accepted after accept command"
        );
        assert_eq!(loaded.version(), 2);
    }

    /// `InvitationCommand::revoke` transitions the invitation to `Revoked`.
    #[tokio::test]
    async fn test_command_revoke_transitions_to_revoked() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let cmd = InvitationCommand::new(repo);
        let id = invitation_id();

        let _created = cmd
            .create(
                id.clone(),
                workspace_id(),
                user_id(),
                "eve@example.com".to_string(),
                role_id(),
                7,
            )
            .await
            .expect("create must succeed");

        cmd.revoke(id.clone()).await.expect("revoke must succeed");

        let repo2 = make_repository(&db).await;
        let loaded = repo2.get(&id).await.expect("get must succeed");
        assert!(
            matches!(loaded.status(), InvitationStatus::Revoked),
            "invitation must be Revoked after revoke command"
        );
        assert_eq!(loaded.version(), 2);
    }

    /// The projector inserts a row into `projections__invitations` on `InvitationCreated`.
    #[tokio::test]
    async fn test_projector_inserts_row_on_invitation_created() {
        let db = TestFixture::setup().await;
        seed_fk_parents(db.admin.as_ref()).await;
        let mut projector = InvitationProjector::new(db.admin.clone());
        let id = invitation_id();

        projector
            .handle(raw_created_event(&id))
            .await
            .expect("projector must handle InvitationCreated");

        let row = sqlx::query("SELECT status FROM projections__invitations WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(db.admin.as_ref())
            .await
            .expect("query must succeed");

        assert!(
            row.is_some(),
            "projection row must exist after InvitationCreated"
        );
        let status: String = row.unwrap().try_get("status").unwrap();
        assert_eq!(status, "pending");
    }

    /// The projector updates the status to `accepted` on `InvitationAccepted`.
    #[tokio::test]
    async fn test_projector_updates_status_on_invitation_accepted() {
        let db = TestFixture::setup().await;
        seed_fk_parents(db.admin.as_ref()).await;
        let mut projector = InvitationProjector::new(db.admin.clone());
        let id = invitation_id();

        projector
            .handle(raw_created_event(&id))
            .await
            .expect("InvitationCreated must be handled");

        let accepted_by: UserId = "019d0ce8-facb-7c90-b9d7-000000000050"
            .parse()
            .expect("valid UUID");
        let accepted_event = InvitationEvent::Accepted { accepted_by };
        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 2,
                global_position: 2,
                event_type: "InvitationAccepted".to_string(),
                payload_bytes: serde_json::to_vec(&accepted_event).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("InvitationAccepted must be handled");

        let row = sqlx::query("SELECT status FROM projections__invitations WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(db.admin.as_ref())
            .await
            .expect("row must exist");
        assert_eq!(row.try_get::<String, _>("status").unwrap(), "accepted");
    }

    /// The projector updates the status to `revoked` on `InvitationRevoked`.
    #[tokio::test]
    async fn test_projector_updates_status_on_invitation_revoked() {
        let db = TestFixture::setup().await;
        seed_fk_parents(db.admin.as_ref()).await;
        let mut projector = InvitationProjector::new(db.admin.clone());
        let id = invitation_id();

        projector
            .handle(raw_created_event(&id))
            .await
            .expect("InvitationCreated must be handled");

        let revoked_event = InvitationEvent::Revoked {};
        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 2,
                global_position: 2,
                event_type: "InvitationRevoked".to_string(),
                payload_bytes: serde_json::to_vec(&revoked_event).unwrap(),
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("InvitationRevoked must be handled");

        let row = sqlx::query("SELECT status FROM projections__invitations WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(db.admin.as_ref())
            .await
            .expect("row must exist");
        assert_eq!(row.try_get::<String, _>("status").unwrap(), "revoked");
    }

    /// `InvitationRepository::find_by_token` resolves a token to an `InvitationRow`.
    #[tokio::test]
    async fn test_find_by_token_returns_row_after_projection() {
        let db = TestFixture::setup().await;
        seed_fk_parents(db.admin.as_ref()).await;
        let mut projector = InvitationProjector::new(db.admin.clone());
        let repo = make_repository(&db).await;
        let id = invitation_id();

        projector
            .handle(raw_created_event(&id))
            .await
            .expect("InvitationCreated must be handled");

        let result = repo
            .find_by_token("test-token-abc")
            .await
            .expect("find_by_token must succeed");

        assert!(result.is_some(), "token must resolve to an InvitationRow");
        let row = result.unwrap();
        assert_eq!(row.email(), "bob@example.com");
        assert!(matches!(row.status, InvitationStatus::Pending));
    }

    /// `find_by_token` returns `None` for an unknown token.
    #[tokio::test]
    async fn test_find_by_token_returns_none_for_unknown_token() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;

        let result = repo
            .find_by_token("does-not-exist")
            .await
            .expect("query must succeed");

        assert!(result.is_none());
    }

    /// The projector is idempotent — sending the same `InvitationCreated` twice must not error.
    #[tokio::test]
    async fn test_projector_is_idempotent() {
        let db = TestFixture::setup().await;
        seed_fk_parents(db.admin.as_ref()).await;
        let mut projector = InvitationProjector::new(db.admin.clone());
        let id = invitation_id();

        projector.handle(raw_created_event(&id)).await.unwrap();
        let result = projector.handle(raw_created_event(&id)).await;
        assert!(
            result.is_ok(),
            "duplicate InvitationCreated must be ignored"
        );
    }

    /// The projector ignores unknown event types without error.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = InvitationProjector::new(db.admin);

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
}
