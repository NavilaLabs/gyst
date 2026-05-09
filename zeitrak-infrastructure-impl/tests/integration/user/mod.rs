use eventually::aggregate::{
    Aggregate,
    repository::{Getter, Saver},
};
use eventually_projection::{Projector, RawEvent};
use zeitrak_core::admin::user::{User, UserEvent, UserId};
use zeitrak_infrastructure_impl::admin::user::{
    projectors::UserProjector, repositories::UserRepository,
};
use zeitrak_tests::TestFixture;
use sqlx::Row;

fn test_id() -> UserId {
    "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
        .parse()
        .expect("valid UUID")
}

async fn make_repository(fixture: &TestFixture) -> UserRepository {
    UserRepository::from_pool(fixture.admin.clone())
        .await
        .expect("UserRepository must be created")
}

pub mod tests {
    use zeitrak_core::admin::user::UserRepository;

    use super::*;

    /// Saving a new aggregate root persists it; loading it back returns the
    /// same state and version.
    #[tokio::test]
    async fn test_save_and_get_user() {
        let db = TestFixture::setup().await;
        let repo = make_repository(&db).await;
        let id = test_id();

        let mut root = eventually::aggregate::Root::<User>::record_new(
            UserEvent::Created {
                id: id.clone(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                password: String::new(),
            }
            .into(),
        )
        .expect("Created event on a new aggregate is always valid");

        repo.save(&mut root).await.expect("save must succeed");

        let loaded = repo.get(&id).await.expect("get must succeed");
        assert_eq!(loaded.aggregate_id(), &id);
        assert_eq!(loaded.name(), "Alice");
        assert_eq!(loaded.version(), 1);
    }

    /// Applying a second `Created` event to an already-existing `User` must
    /// return an `AlreadyExists` error — pure domain logic, no database needed.
    #[test]
    fn test_duplicate_user_creation_is_rejected_by_domain() {
        let id = test_id();
        let existing = User::apply(
            None,
            UserEvent::Created {
                id: id.clone(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                password: String::new(),
            },
        )
        .expect("first Created is valid");

        let result = User::apply(
            Some(existing),
            UserEvent::Created {
                id,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                password: String::new(),
            },
        );
        assert!(
            result.is_err(),
            "second Created on an existing user must fail"
        );
    }

    /// The projector must insert a row into the projection table when it
    /// receives a `UserCreated` event.
    #[tokio::test]
    async fn test_projector_inserts_row_on_user_created() {
        let db = TestFixture::setup().await;
        let mut projector = UserProjector::new(db.admin.clone());

        let id = test_id();
        let event = UserEvent::Created {
            id: id.clone(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            password: String::new(),
        };
        let payload_bytes = serde_json::to_vec(&event).expect("serialization must succeed");

        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 1,
                global_position: 1,
                event_type: "UserCreated".to_string(),
                payload_bytes,
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("projector must handle UserCreated");

        let rows = sqlx::query("SELECT name FROM projections__users")
            .fetch_all(db.admin.as_ref())
            .await
            .expect("query must succeed");

        let found = rows.iter().any(|r| {
            let name: String = r.get("name");
            name == "Alice"
        });
        assert!(found, "projection table should contain a row for Alice");
    }

    /// The projector must silently ignore event types it does not handle.
    #[tokio::test]
    async fn test_projector_ignores_unknown_event_type() {
        let db = TestFixture::setup().await;
        let mut projector = UserProjector::new(db.admin);

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

    /// After a `UserCreated` event the projection table row is queryable
    /// through `UserRepository::find_credentials_by_email`.
    #[tokio::test]
    async fn test_find_credentials_by_email_after_projection() {
        let db = TestFixture::setup().await;
        let mut projector = UserProjector::new(db.admin.clone());
        let repo = make_repository(&db).await;

        let id = test_id();
        let event = UserEvent::Created {
            id: id.clone(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "hashed_pw".to_string(),
        };
        let payload_bytes = serde_json::to_vec(&event).expect("serialization must succeed");

        projector
            .handle(RawEvent {
                stream_id: id.to_string(),
                version: 1,
                global_position: 1,
                event_type: "UserCreated".to_string(),
                payload_bytes,
                metadata: serde_json::Value::Null,
                schema_version: 1,
            })
            .await
            .expect("projector must handle UserCreated");

        let creds = repo
            .find_credentials_by_email("alice@example.com")
            .await
            .expect("query must succeed");

        assert!(
            creds.is_some(),
            "credentials must be found after projection"
        );
        let (found_id, found_email, found_hash) = creds.unwrap();
        assert_eq!(found_id, id.to_string());
        assert_eq!(found_email, "alice@example.com");
        assert_eq!(found_hash, "hashed_pw");
    }
}
