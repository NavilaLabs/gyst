use chrono::{DateTime, Utc};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

/// A cached `OAuth2` access token with its expiry timestamp.
pub struct CachedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Thread-safe cache for the current Microsoft `OAuth2` access token.
pub type TokenCache = Arc<Mutex<Option<CachedToken>>>;

/// Application-global token cache — shared across all email sends.
pub static TOKEN_CACHE: LazyLock<TokenCache> = LazyLock::new(|| Arc::new(Mutex::new(None)));
