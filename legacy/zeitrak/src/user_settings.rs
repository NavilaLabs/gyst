use anyhow::Result;
use zeitrak_core::admin::user::{
    UserCommand, UserCommandTrait, UserId, UserQuery, UserQueryTrait, UserRow,
};
use zeitrak_infrastructure_impl::{Pool, admin::user::repositories::UserRepository};

/// Returns the current settings for the given user.
pub async fn get_user_settings(user_id: &str) -> Result<UserRow> {
    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool).await?;
    UserQuery::new(repo)
        .find_view_by_id(user_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("user not found"))
}

/// Records a `UserSettingsUpdated` event for the given user.
pub async fn update_user_settings(
    user_id: &str,
    timezone: String,
    date_format: String,
    language: String,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool).await?;
    let agg_id: UserId = user_id.parse()?;
    UserCommand::new(repo)
        .update_settings(agg_id, timezone, date_format, language)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
