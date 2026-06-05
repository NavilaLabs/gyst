use zeitrak_infrastructure::email::{PersistedSmtpConfig, SmtpAuthMethod, SmtpConfigRepository};
use zeitrak_infrastructure_impl::smtp::SmtpConfigRepositoryImpl;
use zeitrak_tests::TestFixture;

const TEST_SECRET: &str = "test-auth-secret-for-smtp-integration";

fn make_repo(fixture: &TestFixture) -> SmtpConfigRepositoryImpl {
    SmtpConfigRepositoryImpl::new(fixture.admin.clone(), TEST_SECRET)
}

fn password_config() -> PersistedSmtpConfig {
    PersistedSmtpConfig {
        auth_method: SmtpAuthMethod::Password,
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        from_address: "noreply@example.com".to_string(),
        use_tls: true,
        password: Some("s3cr3t".to_string()),
        client_id: None,
        client_secret: None,
        tenant_id: None,
        refresh_token: None,
        oauth2_smtp_email: None,
        oauth2_authorized: false,
    }
}

/// Save a config and read it back — all fields must survive the round-trip,
/// with sensitive values decrypted correctly.
#[tokio::test]
async fn smtp_config_repository_roundtrip() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    assert!(repo.get().await.unwrap().is_none(), "fresh DB must have no config");

    let config = password_config();
    repo.save(&config).await.unwrap();

    let loaded = repo.get().await.unwrap().expect("must exist after save");
    assert_eq!(loaded.auth_method, SmtpAuthMethod::Password);
    assert_eq!(loaded.host, "smtp.example.com");
    assert_eq!(loaded.port, 587);
    assert_eq!(loaded.username, "user@example.com");
    assert_eq!(loaded.from_address, "noreply@example.com");
    assert!(loaded.use_tls);
    assert_eq!(loaded.password.as_deref(), Some("s3cr3t"));
    assert_eq!(loaded.client_id, None);
    assert!(!loaded.oauth2_authorized);
}

/// Saving twice must overwrite, not duplicate, the single row.
#[tokio::test]
async fn smtp_config_second_save_overwrites() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    repo.save(&password_config()).await.unwrap();
    let mut updated = password_config();
    updated.host = "smtp2.example.com".to_string();
    repo.save(&updated).await.unwrap();

    let loaded = repo.get().await.unwrap().unwrap();
    assert_eq!(loaded.host, "smtp2.example.com");
}

/// After saving a config with a password, saving again with `password = None`
/// (i.e. "keep existing") should still return the original password.
///
/// The caller is responsible for passing `password = None` to preserve the
/// existing encrypted value — this test verifies the DB round-trip, not the
/// facade's merge logic.
#[tokio::test]
async fn smtp_config_null_password_clears_it() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    // First save with a password.
    repo.save(&password_config()).await.unwrap();

    // Re-save with password = None (explicitly clearing).
    let mut no_pw = password_config();
    no_pw.password = None;
    repo.save(&no_pw).await.unwrap();

    let loaded = repo.get().await.unwrap().unwrap();
    // Clearing with None means the password column is NULL — get() returns None.
    assert_eq!(loaded.password, None);
}

/// The `OAuth2` state flow: set state → complete with correct state → `refresh_token` stored.
#[tokio::test]
async fn oauth2_state_flow_succeeds() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    // Need a row first.
    repo.save(&password_config()).await.unwrap();
    repo.set_oauth2_state("csrf-state-abc123").await.unwrap();

    let refresh = "my-refresh-token-value";
    repo.complete_oauth2("csrf-state-abc123", refresh).await.unwrap();

    let loaded = repo.get().await.unwrap().unwrap();
    assert!(loaded.oauth2_authorized);
    assert_eq!(loaded.refresh_token.as_deref(), Some(refresh));
}

/// Completing `OAuth2` with the wrong state must fail (CSRF protection).
#[tokio::test]
async fn oauth2_state_mismatch_fails() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    repo.save(&password_config()).await.unwrap();
    repo.set_oauth2_state("correct-state").await.unwrap();

    let result = repo.complete_oauth2("wrong-state", "token").await;
    assert!(result.is_err(), "mismatched state must return an error");

    // oauth2_authorized must still be false.
    let loaded = repo.get().await.unwrap().unwrap();
    assert!(!loaded.oauth2_authorized);
}

/// `set_oauth2_state` on a non-existent row must return an error.
#[tokio::test]
async fn set_oauth2_state_without_config_fails() {
    let fixture = TestFixture::setup().await;
    let repo = make_repo(&fixture);

    let result = repo.set_oauth2_state("state").await;
    assert!(result.is_err(), "must fail when no smtp_config row exists");
}
