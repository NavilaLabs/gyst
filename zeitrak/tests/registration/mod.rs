use zeitrak::registration::{register_user_on, verify_email_by_token_on};
use zeitrak_tests::{
    RecordingEmailSender, SentEmailKind, TestFixture, flush_user_projector,
    is_user_email_verified,
};

/// `register_user_on` must send exactly one verification email to the
/// registrant's address, and the link must contain the verification token path.
#[tokio::test]
async fn register_user_sends_verification_email() {
    let db = TestFixture::setup().await;
    let sender = RecordingEmailSender::new();

    let result = register_user_on(
        db.admin.clone(),
        "Alice".to_string(),
        "alice@example.com".to_string(),
        "Password1!".to_string(),
        &sender,
        "http://localhost:8080",
    )
    .await;

    assert!(result.is_ok(), "register_user_on must succeed: {result:?}");

    let sent = sender.sent();
    assert_eq!(sent.len(), 1, "exactly one email must be sent on registration");
    assert_eq!(sent[0].to, "alice@example.com");

    let SentEmailKind::Verification { verification_link } = &sent[0].kind else {
        panic!("expected a Verification email, got {:?}", sent[0].kind);
    };
    assert!(
        verification_link.starts_with("http://localhost:8080/verify-email/"),
        "link must start with base_url/verify-email/: {verification_link}"
    );
}

/// Re-registering an unverified email must resend a fresh verification email
/// rather than returning an error, and must return the same user id.
#[tokio::test]
async fn register_user_unverified_duplicate_resends_verification_email() {
    let db = TestFixture::setup().await;
    let sender = RecordingEmailSender::new();

    let first_id = register_user_on(
        db.admin.clone(),
        "Alice".to_string(),
        "alice@example.com".to_string(),
        "Password1!".to_string(),
        &sender,
        "http://localhost:8080",
    )
    .await
    .expect("first registration must succeed");

    // Flush so the projection table reflects the unverified account.
    flush_user_projector(&db.admin).await;

    let sender2 = RecordingEmailSender::new();
    let second_id = register_user_on(
        db.admin.clone(),
        "Alice".to_string(),
        "alice@example.com".to_string(),
        "Password1!".to_string(),
        &sender2,
        "http://localhost:8080",
    )
    .await
    .expect("re-registration of an unverified account must succeed");

    assert_eq!(
        second_id, first_id,
        "re-registration must return the same user id"
    );

    let sent = sender2.sent();
    assert_eq!(sent.len(), 1, "exactly one new verification email must be sent");
    assert_eq!(sent[0].to, "alice@example.com");

    let SentEmailKind::Verification { verification_link } = &sent[0].kind else {
        panic!("expected a Verification email, got {:?}", sent[0].kind);
    };
    assert!(
        verification_link.starts_with("http://localhost:8080/verify-email/"),
        "fresh link must start with base_url/verify-email/: {verification_link}"
    );
}

/// Re-registering an already-verified email must fail without sending any email.
#[tokio::test]
async fn register_user_verified_duplicate_sends_no_email() {
    let db = TestFixture::setup().await;
    let sender = RecordingEmailSender::new();

    register_user_on(
        db.admin.clone(),
        "Alice".to_string(),
        "alice@example.com".to_string(),
        "Password1!".to_string(),
        &sender,
        "http://localhost:8080",
    )
    .await
    .expect("first registration must succeed");

    flush_user_projector(&db.admin).await;

    let token = {
        let sent = sender.sent();
        let SentEmailKind::Verification { verification_link } = &sent[0].kind else {
            panic!("expected Verification email");
        };
        verification_link
            .rsplit('/')
            .next()
            .expect("link must contain a token segment")
            .to_string()
    };

    zeitrak::registration::verify_email_by_token_on(db.admin.clone(), &token)
        .await
        .expect("verification must succeed");

    flush_user_projector(&db.admin).await;

    let sender2 = RecordingEmailSender::new();
    let result = register_user_on(
        db.admin.clone(),
        "AliceAgain".to_string(),
        "alice@example.com".to_string(),
        "Password1!".to_string(),
        &sender2,
        "http://localhost:8080",
    )
    .await;

    assert!(result.is_err(), "re-registration of a verified account must fail");
    assert_eq!(sender2.sent().len(), 0, "no email must be sent on verified duplicate");
}

/// Full registration flow: register → receive verification email → click the
/// link → account is marked verified in the projection.
///
/// This mirrors what a real user does: register, open the email, click the
/// link (which contains the token), and land on the confirmation page.
#[tokio::test]
async fn register_and_verify_email_via_link() {
    let db = TestFixture::setup().await;
    let sender = RecordingEmailSender::new();

    // 1. Register a new user.
    let user_id = register_user_on(
        db.admin.clone(),
        "Dave".to_string(),
        "dave@example.com".to_string(),
        "Password1!".to_string(),
        &sender,
        "http://localhost:8080",
    )
    .await
    .expect("registration must succeed");

    // 2. The registration sends exactly one verification email.
    let sent = sender.sent();
    assert_eq!(sent.len(), 1, "exactly one verification email must be sent");
    assert_eq!(sent[0].to, "dave@example.com");

    let SentEmailKind::Verification { verification_link } = &sent[0].kind else {
        panic!("expected a Verification email, got {:?}", sent[0].kind);
    };

    // 3. Simulate clicking the link: extract the token from the URL.
    let token = verification_link
        .rsplit('/')
        .next()
        .expect("verification link must contain a token path segment");

    assert!(
        !token.is_empty(),
        "token extracted from {verification_link} must not be empty"
    );

    // 4. Flush events so the token is visible in the projection.
    flush_user_projector(&db.admin).await;

    // 5. Use the token to verify the email address.
    let verified_id = verify_email_by_token_on(db.admin.clone(), token)
        .await
        .expect("email verification with the token from the email must succeed");

    assert_eq!(
        verified_id, user_id,
        "verify_email_by_token_on must return the same user id as registration"
    );

    // 6. Flush again so the Verified event is reflected in the projection.
    flush_user_projector(&db.admin).await;

    // 7. The user must now appear as verified in the read model.
    assert!(
        is_user_email_verified(&db.admin, &user_id.to_string()).await,
        "user must be marked as verified in the projection after clicking the link"
    );
}

/// The token in the verification link must be valid exactly once.
#[tokio::test]
async fn verify_email_by_token_succeeds_and_is_single_use() {
    let db = TestFixture::setup().await;
    let sender = RecordingEmailSender::new();

    register_user_on(
        db.admin.clone(),
        "Carol".to_string(),
        "carol@example.com".to_string(),
        "Password1!".to_string(),
        &sender,
        "http://localhost:8080",
    )
    .await
    .expect("registration must succeed");

    // Flush events into the projection so the verification token is queryable.
    flush_user_projector(&db.admin).await;

    let sent = sender.sent();
    let SentEmailKind::Verification { verification_link } = &sent[0].kind else {
        panic!("expected Verification email");
    };
    let token = verification_link
        .rsplit('/')
        .next()
        .expect("link must contain a token segment");

    let first = verify_email_by_token_on(db.admin.clone(), token).await;
    assert!(first.is_ok(), "first verification must succeed: {first:?}");

    // Flush again so the Verified event clears the token from the projection.
    flush_user_projector(&db.admin).await;

    let second = verify_email_by_token_on(db.admin.clone(), token).await;
    assert!(second.is_err(), "token must be invalid after first use");
}
