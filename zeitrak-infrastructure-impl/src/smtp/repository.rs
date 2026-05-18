use async_trait::async_trait;
use chrono::Utc;
use zeitrak_infrastructure::email::{PersistedSmtpConfig, SmtpAuthMethod, SmtpConfigRepository};

use crate::{ConnectedAdminPool, smtp::encryption};

/// Reads and writes the single-row `smtp_config` table in the admin database.
///
/// Sensitive fields (`password`, `client_secret`, `refresh_token`) are
/// transparently encrypted/decrypted with AES-256-GCM.
pub struct SmtpConfigRepositoryImpl {
    pool: ConnectedAdminPool,
    key: [u8; 32],
}

impl SmtpConfigRepositoryImpl {
    /// Creates a new repository instance.
    ///
    /// `auth_secret` is typically `CONFIG.application().security().authentication_secret()`.
    /// The encryption key is derived from it via SHA-256.
    #[must_use]
    pub fn new(pool: ConnectedAdminPool, auth_secret: &str) -> Self {
        Self {
            pool,
            key: encryption::derive_key(auth_secret),
        }
    }
}

#[async_trait]
impl SmtpConfigRepository for SmtpConfigRepositoryImpl {
    async fn get(&self) -> anyhow::Result<Option<PersistedSmtpConfig>> {
        let row = sqlx::query(
            "SELECT auth_method, host, port, username, from_address, use_tls,
                    encrypted_password, password_nonce,
                    oauth2_client_id, oauth2_tenant_id, oauth2_smtp_email,
                    encrypted_client_secret, client_secret_nonce,
                    encrypted_refresh_token, refresh_token_nonce,
                    oauth2_authorized
             FROM smtp_config WHERE id = 1",
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        use sqlx::Row as _;

        let auth_method_str: String = row.try_get("auth_method")?;
        let auth_method = if auth_method_str == "xoauth2" {
            SmtpAuthMethod::XOAuth2
        } else {
            SmtpAuthMethod::Password
        };

        let port_i64: i64 = row.try_get("port")?;
        let use_tls_i64: i64 = row.try_get("use_tls")?;
        let oauth2_authorized_i64: i64 = row.try_get("oauth2_authorized")?;

        let password = decrypt_optional(
            &self.key,
            row.try_get("encrypted_password")?,
            row.try_get("password_nonce")?,
        )?;

        let client_secret = decrypt_optional(
            &self.key,
            row.try_get("encrypted_client_secret")?,
            row.try_get("client_secret_nonce")?,
        )?;

        let refresh_token = decrypt_optional(
            &self.key,
            row.try_get("encrypted_refresh_token")?,
            row.try_get("refresh_token_nonce")?,
        )?;

        Ok(Some(PersistedSmtpConfig {
            auth_method,
            host: row.try_get("host")?,
            port: u16::try_from(port_i64).unwrap_or(587),
            username: row.try_get("username")?,
            from_address: row.try_get("from_address")?,
            use_tls: use_tls_i64 != 0,
            password,
            client_id: row.try_get("oauth2_client_id")?,
            client_secret,
            tenant_id: row.try_get("oauth2_tenant_id")?,
            refresh_token,
            oauth2_smtp_email: row.try_get("oauth2_smtp_email")?,
            oauth2_authorized: oauth2_authorized_i64 != 0,
        }))
    }

    async fn save(&self, config: &PersistedSmtpConfig) -> anyhow::Result<()> {
        let auth_method = match config.auth_method {
            SmtpAuthMethod::Password => "password",
            SmtpAuthMethod::XOAuth2 => "xoauth2",
            _ => "password",
        };
        let now = Utc::now().to_rfc3339();
        let use_tls: i64 = i64::from(config.use_tls);

        let (enc_password, pw_nonce) =
            encrypt_optional(&self.key, config.password.as_deref())?;
        let (enc_secret, secret_nonce) =
            encrypt_optional(&self.key, config.client_secret.as_deref())?;
        let (enc_refresh, refresh_nonce) =
            encrypt_optional(&self.key, config.refresh_token.as_deref())?;

        let oauth2_authorized: i64 = i64::from(config.oauth2_authorized);

        sqlx::query(
            "INSERT OR REPLACE INTO smtp_config (
                id, auth_method, host, port, username, from_address, use_tls,
                encrypted_password, password_nonce,
                oauth2_client_id, oauth2_tenant_id, oauth2_smtp_email,
                encrypted_client_secret, client_secret_nonce,
                encrypted_refresh_token, refresh_token_nonce,
                oauth2_authorized, updated_at
            ) VALUES (
                1, ?, ?, ?, ?, ?, ?,
                ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?,
                ?, ?
            )",
        )
        .bind(auth_method)
        .bind(&config.host)
        .bind(i64::from(config.port))
        .bind(&config.username)
        .bind(&config.from_address)
        .bind(use_tls)
        .bind(enc_password)
        .bind(pw_nonce)
        .bind(config.client_id.as_deref())
        .bind(config.tenant_id.as_deref())
        .bind(config.oauth2_smtp_email.as_deref())
        .bind(enc_secret)
        .bind(secret_nonce)
        .bind(enc_refresh)
        .bind(refresh_nonce)
        .bind(oauth2_authorized)
        .bind(&now)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn set_oauth2_state(&self, state: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = sqlx::query(
            "UPDATE smtp_config SET oauth2_state = ?, updated_at = ? WHERE id = 1",
        )
        .bind(state)
        .bind(&now)
        .execute(self.pool.as_ref())
        .await?
        .rows_affected();

        anyhow::ensure!(
            rows_affected > 0,
            "no smtp_config row exists — save a config before starting OAuth2"
        );
        Ok(())
    }

    async fn complete_oauth2(&self, actual_state: &str, refresh_token: &str) -> anyhow::Result<()> {
        // Read the stored CSRF state to validate
        let row = sqlx::query("SELECT oauth2_state FROM smtp_config WHERE id = 1")
            .fetch_optional(self.pool.as_ref())
            .await?;

        let row = row.ok_or_else(|| anyhow::anyhow!("no smtp_config row exists"))?;

        use sqlx::Row as _;
        let stored_state: Option<String> = row.try_get("oauth2_state")?;
        let stored_state = stored_state.ok_or_else(|| anyhow::anyhow!("no OAuth2 state set"))?;

        anyhow::ensure!(
            stored_state == actual_state,
            "OAuth2 state mismatch — possible CSRF attack"
        );

        let (enc_refresh, refresh_nonce) =
            encryption::encrypt(&self.key, refresh_token)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE smtp_config
             SET encrypted_refresh_token = ?, refresh_token_nonce = ?,
                 oauth2_authorized = 1, oauth2_state = NULL, updated_at = ?
             WHERE id = 1",
        )
        .bind(enc_refresh)
        .bind(refresh_nonce)
        .bind(&now)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

fn decrypt_optional(
    key: &[u8; 32],
    ciphertext: Option<String>,
    nonce: Option<String>,
) -> anyhow::Result<Option<String>> {
    match (ciphertext, nonce) {
        (Some(ct), Some(n)) if !ct.is_empty() => {
            encryption::decrypt(key, &ct, &n).map(Some)
        }
        _ => Ok(None),
    }
}

fn encrypt_optional(
    key: &[u8; 32],
    plaintext: Option<&str>,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    match plaintext {
        Some(p) if !p.is_empty() => {
            let (ct, nonce) = encryption::encrypt(key, p)?;
            Ok((Some(ct), Some(nonce)))
        }
        _ => Ok((None, None)),
    }
}
