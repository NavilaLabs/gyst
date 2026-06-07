use anyhow::Result;
use serde::Deserialize;
use zeitrak_infrastructure::{config::CONFIG, email::SmtpConfigRepository as _};
use zeitrak_infrastructure_impl::{Pool, smtp::SmtpConfigRepositoryImpl};

async fn build_repo() -> Result<SmtpConfigRepositoryImpl> {
    let pool = Pool::connect_admin().await?;
    let secret = CONFIG.application().security().authentication_secret();
    Ok(SmtpConfigRepositoryImpl::new(pool, secret))
}

/// Initiates the Microsoft `OAuth2` authorization code flow.
///
/// Stores a random CSRF state in the admin database and returns the full
/// authorization URL for the user to open in their browser.
///
/// # Errors
///
/// Returns an error if no SMTP config row exists yet or the state cannot be saved.
pub async fn initiate_microsoft_oauth2(client_id: &str, tenant_id: &str) -> Result<String> {
    use rand::Rng as _;
    let state: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let repo = build_repo().await?;
    repo.set_oauth2_state(&state).await?;

    let base_url = CONFIG.application().base_url();
    let redirect_uri = format!("{base_url}/api/smtp/oauth2/callback");
    let scope = "https://outlook.office.com/SMTP.Send offline_access";

    let url = format!(
        "https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/authorize\
         ?client_id={client_id}\
         &response_type=code\
         &redirect_uri={encoded_redirect}\
         &scope={encoded_scope}\
         &state={state}\
         &prompt=select_account",
        encoded_redirect = urlencoding::encode(&redirect_uri),
        encoded_scope = urlencoding::encode(scope),
    );

    Ok(url)
}

/// Partial token response from the Microsoft token endpoint.
#[derive(Deserialize)]
struct TokenResponse {
    refresh_token: String,
}

/// Handles the `OAuth2` authorization callback.
///
/// Exchanges `code` for tokens and persists the `refresh_token` in the admin
/// database.  The CSRF `state` is validated against the stored value.
///
/// # Errors
///
/// Returns an error if the state is invalid, the token exchange fails, or the
/// database write fails.
pub async fn complete_microsoft_oauth2(code: String, state: String) -> Result<()> {
    let repo = build_repo().await?;

    let existing = repo
        .get()
        .await?
        .ok_or_else(|| anyhow::anyhow!("no smtp_config row — cannot complete OAuth2 callback"))?;

    let client_id = existing
        .client_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("oauth2 client_id not set"))?
        .to_string();
    let client_secret = existing
        .client_secret
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("oauth2 client_secret not set"))?
        .to_string();
    let tenant_id = existing
        .tenant_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("oauth2 tenant_id not set"))?
        .to_string();

    let base_url = CONFIG.application().base_url();
    let redirect_uri = format!("{base_url}/api/smtp/oauth2/callback");

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        (
            "scope",
            "https://outlook.office.com/SMTP.Send offline_access",
        ),
    ];

    let token_url = format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token");

    let http = reqwest::Client::new();
    let resp = http
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("token request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Microsoft token endpoint returned {status}: {body}");
    }

    let token: TokenResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("failed to parse token response: {e}"))?;

    repo.complete_oauth2(&state, &token.refresh_token).await?;
    Ok(())
}

/// Returns `true` if the `OAuth2` authorization flow has completed successfully.
///
/// # Errors
///
/// Returns an error if the admin database cannot be reached.
pub async fn oauth2_status() -> Result<bool> {
    let repo = build_repo().await?;
    Ok(repo.get().await?.is_some_and(|c| c.oauth2_authorized))
}
