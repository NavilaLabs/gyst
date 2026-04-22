use anyhow::Result;
use eventually::aggregate::repository::{Getter, Saver};
use loom_core::admin::user::{UserCommand, UserId, UserRow};
use loom_infrastructure_impl::{Pool, admin::user::repositories::UserRepository};

/// Returns the current settings for the given user.
pub async fn get_user_settings(user_id: &str) -> Result<UserRow> {
    let pool = Pool::connect_admin().await?;
    let repo = UserRepository::from_pool(pool).await?;
    repo.find_view_by_id(user_id)
        .await?
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
    let root = repo
        .get(&agg_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut cmd: UserCommand = root.into();
    cmd.update_settings(timezone, date_format, language)?;
    repo.save(&mut cmd)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
