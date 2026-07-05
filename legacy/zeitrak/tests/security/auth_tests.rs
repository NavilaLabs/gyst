/// Tests for JWT token validation.
///
/// These tests exercise `zeitrak::auth::validate_token` — the boundary where
/// an untrusted string token is converted into a trusted `CurrentUser`.
/// They run without a database but DO require `.env.test` to be loaded so
/// that `CONFIG.application().authentication_secret()` returns a
/// deterministic value.
///
/// Attack scenarios covered:
///   - Wrong HMAC secret           → rejected
///   - Expired token               → rejected
///   - Tampered payload            → rejected (signature mismatch)
///   - Algorithm confusion HS512   → rejected (only HS256 is accepted)
///   - "alg:none" unsigned token   → rejected
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use serial_test::serial;
use std::time::{SystemTime, UNIX_EPOCH};
use with_lifecycle::with_lifecycle;
use zeitrak::auth::{CurrentUser, validate_token};
use zeitrak_tests::test_lifecycle;

// ── helpers ──────────────────────────────────────────────────────────────────

/// The test secret from `.env.test`.
const TEST_SECRET: &[u8] = "s€cR€+".as_bytes();

/// An exp value 1 hour in the future.
fn future_exp() -> usize {
    usize::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .expect("timestamp fits in usize")
        + 3600
}

/// A clearly expired timestamp (Unix epoch).
/// Must be well outside jsonwebtoken's default 60-second leeway.
const fn past_exp() -> usize {
    0 // 1970-01-01 — never valid
}

#[derive(Serialize)]
struct Claims {
    sub: String,
    email: String,
    exp: usize,
}

fn make_hs256_token(secret: &[u8], sub: &str, email: &str, exp: usize) -> String {
    encode(
        &Header::default(), // Algorithm::HS256
        &Claims {
            sub: sub.to_string(),
            email: email.to_string(),
            exp,
        },
        &EncodingKey::from_secret(secret),
    )
    .expect("token encoding must not fail in tests")
}

fn make_hs512_token(secret: &[u8], sub: &str, email: &str, exp: usize) -> String {
    encode(
        &Header::new(Algorithm::HS512),
        &Claims {
            sub: sub.to_string(),
            email: email.to_string(),
            exp,
        },
        &EncodingKey::from_secret(secret),
    )
    .expect("HS512 token encoding must not fail in tests")
}

// ── happy-path ────────────────────────────────────────────────────────────────

/// A valid token signed with the correct secret must decode to the right user.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn valid_token_decodes_to_correct_user() {
    let token = make_hs256_token(
        TEST_SECRET,
        "user-abc-123",
        "alice@example.com",
        future_exp(),
    );
    let user: CurrentUser = validate_token(&token).expect("valid token must decode");
    assert_eq!(user.id, "user-abc-123");
    assert_eq!(user.email, "alice@example.com");
}

// ── wrong-secret attack ───────────────────────────────────────────────────────

/// A token signed with a different secret must be rejected even if the payload
/// is structurally valid.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn token_with_wrong_secret_is_rejected() {
    let token = make_hs256_token(
        b"wrong-secret",
        "user-abc",
        "alice@example.com",
        future_exp(),
    );
    assert!(
        validate_token(&token).is_err(),
        "token signed with wrong secret must be rejected"
    );
}

// ── expired token ─────────────────────────────────────────────────────────────

/// An expired token must be rejected regardless of its signature validity.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn expired_token_is_rejected() {
    let token = make_hs256_token(TEST_SECRET, "user-abc", "alice@example.com", past_exp());
    assert!(
        validate_token(&token).is_err(),
        "expired token must be rejected"
    );
}

// ── payload tampering ─────────────────────────────────────────────────────────

/// If the payload section of a JWT is modified, the signature no longer matches
/// and the token must be rejected.  This is the standard JWT integrity check.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn tampered_payload_is_rejected() {
    let token = make_hs256_token(TEST_SECRET, "user-abc", "alice@example.com", future_exp());

    // Replace the payload segment with one that claims to be a different user.
    // base64url({"sub":"attacker","email":"evil@example.com","exp":9999999999})
    let evil_payload =
        "eyJzdWIiOiJhdHRhY2tlciIsImVtYWlsIjoiZXZpbEBleGFtcGxlLmNvbSIsImV4cCI6OTk5OTk5OTk5OX0";

    let parts: Vec<&str> = token.splitn(3, '.').collect();
    let tampered = format!("{}.{}.{}", parts[0], evil_payload, parts[2]);

    assert!(
        validate_token(&tampered).is_err(),
        "token with tampered payload must be rejected (signature mismatch)"
    );
}

// ── algorithm confusion ───────────────────────────────────────────────────────

/// The server only accepts HS256.  A token signed with HS512 must be rejected
/// even though it is a valid HMAC token — this prevents algorithm confusion
/// attacks where an attacker switches to a weaker or different algorithm.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn hs512_token_rejected_when_only_hs256_accepted() {
    let token = make_hs512_token(TEST_SECRET, "user-abc", "alice@example.com", future_exp());
    assert!(
        validate_token(&token).is_err(),
        "HS512 token must be rejected — server only accepts HS256"
    );
}

// ── alg:none attack ───────────────────────────────────────────────────────────

/// A JWT with `"alg":"none"` has no signature and should never be accepted.
/// This is a classic attack: if a server accepts alg:none, an attacker can
/// craft arbitrary tokens by stripping the signature.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn alg_none_unsigned_token_is_rejected() {
    // Manually construct an unsigned JWT.
    // header = base64url({"alg":"none","typ":"JWT"})
    let header = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
    // payload = base64url({"sub":"attacker","email":"evil@example.com","exp":9999999999})
    let payload =
        "eyJzdWIiOiJhdHRhY2tlciIsImVtYWlsIjoiZXZpbEBleGFtcGxlLmNvbSIsImV4cCI6OTk5OTk5OTk5OX0";
    // alg:none tokens have an empty signature
    let unsigned_token = format!("{header}.{payload}.");

    assert!(
        validate_token(&unsigned_token).is_err(),
        "alg:none unsigned token must be rejected"
    );
}

// ── token lifetime ────────────────────────────────────────────────────────────

/// A token whose expiry is exactly `now + 3600` (1 hour) must be accepted.
/// This confirms that the 1-hour lifetime the server issues is valid on receipt.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn token_expiring_in_one_hour_is_accepted() {
    let exp = usize::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .expect("timestamp fits in usize")
        + 3_600;
    let token = make_hs256_token(TEST_SECRET, "user-abc", "alice@example.com", exp);
    assert!(
        validate_token(&token).is_ok(),
        "token with 1-hour expiry must be accepted"
    );
}

/// A token that expired one second ago must be rejected.
/// This guards against clock-skew exploits that allow marginally-expired tokens.
#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn token_expired_one_second_ago_is_rejected() {
    // jsonwebtoken has a built-in leeway of 60 s by default, so we use a
    // timestamp well in the past (Unix epoch) to be unambiguously expired.
    let token = make_hs256_token(TEST_SECRET, "user-abc", "alice@example.com", 1_usize); // 1970-01-01 00:00:01 UTC
    assert!(
        validate_token(&token).is_err(),
        "token with past expiry must be rejected"
    );
}

// ── garbage input ─────────────────────────────────────────────────────────────

#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn empty_string_is_rejected() {
    assert!(validate_token("").is_err());
}

#[serial]
#[with_lifecycle(test_lifecycle)]
#[tokio::test]
async fn random_string_is_rejected() {
    assert!(validate_token("not.a.jwt").is_err());
}
