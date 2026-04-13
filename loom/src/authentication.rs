use anyhow::Result;
use loom_core::admin::{authenticator::Authenticator, user::LoginQuery};
use loom_infrastructure::config::CONFIG;
use loom_infrastructure_impl::{
    Pool,
    admin::{
        authentication::jwt::JwtAuthentication, user::repositories::UserRepository,
        workspace::repositories::WorkspaceRepository,
    },
};
use serde::{Deserialize, Serialize};

/// Identity extracted from a validated JWT.  Carries no permissions —
/// those are always checked live against the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: String,
    pub email: String,
}

/// Authenticate with email + password.  Returns a signed JWT on success.
pub async fn login_user(email: String, password: String) -> Result<String> {
    let pool = Pool::connect_admin().await?;
    let user_repo = UserRepository::from_pool(pool).await?;
    let query = LoginQuery::new(user_repo, Authenticator::new(JwtAuthentication));
    let token = query
        .login(&email, &password)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(token)
}

/// Validate a JWT and extract the [`CurrentUser`] it represents.
///
/// Returns an error if the token is malformed, expired, or signed with the
/// wrong secret.
pub fn validate_token(token: &str) -> Result<CurrentUser> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: String,
        email: String,
        exp: usize,
    }

    let secret = CONFIG.get_application().get_authentication_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(CurrentUser {
        id: token_data.claims.sub,
        email: token_data.claims.email,
    })
}

/// Returns the first workspace ID the given user belongs to, or `None`.
pub async fn get_user_workspace(user_id: &str) -> Result<Option<String>> {
    let pool = Pool::connect_admin().await?;
    let workspace_repo = WorkspaceRepository::from_pool(pool).await?;
    Ok(workspace_repo.find_workspace_for_user(user_id).await?)
}
