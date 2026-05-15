use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use zeitrak_core::admin::{
    permission::{PermissionId, PermissionRow},
    user::UserId,
    workspace::{
        MemberRow, WorkspaceCommand, WorkspaceCommandTrait, WorkspaceId, WorkspaceQuery,
        WorkspaceQueryTrait, WorkspaceRepository as WorkspaceRepositoryTrait, WorkspaceRow,
    },
    workspace_role::{
        WorkspaceRoleCommand, WorkspaceRoleCommandTrait, WorkspaceRoleId, WorkspaceRoleRow,
        WorkspaceRoleWithPermissionsRow,
    },
};
use zeitrak_core::admin::workspace_role::WorkspaceRoleRepository as WorkspaceRoleRepositoryTrait;
use zeitrak_core::shared::repositories::ReadRepository;
use zeitrak_infrastructure::database::Migrate;
use zeitrak_infrastructure_impl::{
    Pool, ScopeDefault, ScopeTenant, StateDisconnected,
    admin::{
        permission::repositories::PermissionRepository,
        workspace::repositories::WorkspaceRepository,
        workspace_role::repositories::WorkspaceRoleRepository,
    },
    database::{Initializer, SqliteInitializationStrategy},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: Option<String>,
}

/// Creates a new workspace for an existing user and initialises its tenant database.
///
/// Creates the workspace, default roles (admin + standard), seeds all permissions onto
/// the admin role, seeds the standard set of permissions onto the standard role, and
/// initialises the tenant SQLite database.
pub async fn create_workspace_for_user(
    user_id: UserId,
    workspace_name: String,
) -> Result<WorkspaceId> {
    let pool = Pool::connect_admin().await?;

    let workspace_id = WorkspaceId::new();
    let _ = WorkspaceCommand::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .create(workspace_id.clone(), Some(workspace_name))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Load all seeded permissions for role assignment below.
    let perm_repo = PermissionRepository::from_pool(pool.clone()).await?;
    let all_perms: Vec<_> = perm_repo
        .all()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .into_iter()
        .map(|r| (r.id().clone(), r.name().to_string()))
        .collect();

    let perm_id_for = |name: &str| -> Result<PermissionId> {
        all_perms
            .iter()
            .find(|(_, n)| n == name)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow::anyhow!("permission '{name}' not seeded in database"))
    };

    // --- Admin role: all permissions ---
    let admin_role_id = WorkspaceRoleId::new();
    let role_cmd = WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?);
    let _ = role_cmd
        .create(
            admin_role_id.clone(),
            workspace_id.clone(),
            Some("admin".to_string()),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    for (perm_id, _) in &all_perms {
        WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?)
            .grant_permission(admin_role_id.clone(), perm_id.clone())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool.clone()).await?)
        .assign_user_role(workspace_id.clone(), user_id, admin_role_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // --- Standard role: limited permissions ---
    let standard_role_id = WorkspaceRoleId::new();
    let _ = WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?)
        .create(
            standard_role_id.clone(),
            workspace_id.clone(),
            Some("standard".to_string()),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let standard_permissions = [
        zeitrak_core::permissions::TIMESHEET_CREATE,
        zeitrak_core::permissions::TIMESHEET_UPDATE,
        zeitrak_core::permissions::TIMESHEET_EXPORT,
        zeitrak_core::permissions::TIMESHEET_CANCEL,
    ];

    for perm_name in standard_permissions {
        if let Ok(perm_id) = perm_id_for(perm_name) {
            WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool.clone()).await?)
                .grant_permission(standard_role_id.clone(), perm_id)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
    }

    let default_pool = Pool::<ScopeDefault, StateDisconnected>::connect_default().await?;
    Initializer::new(SqliteInitializationStrategy)
        .initialize_tenant(&default_pool, Some(&workspace_id.to_string()))
        .await?;
    let tenant_pool =
        Pool::<ScopeTenant, StateDisconnected>::connect_tenant(&workspace_id.to_string()).await?;
    tenant_pool.migrate_database().await?;

    Ok(workspace_id)
}

/// Returns all workspaces the given user is a member of.
pub async fn list_user_workspaces(user_id: &str) -> Result<Vec<WorkspaceInfo>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    let rows = WorkspaceQuery::new(repo)
        .find_workspaces_for_user(user_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(rows
        .into_iter()
        .map(|(id, name)| WorkspaceInfo { id, name })
        .collect())
}

/// Returns the current settings for the given workspace.
pub async fn get_workspace_settings(workspace_id: &str) -> Result<WorkspaceRow> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    WorkspaceQuery::new(repo)
        .find_view_by_id(workspace_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("workspace not found"))
}

/// Returns all workspace roles for the given workspace.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list_workspace_roles(workspace_id: &str) -> Result<Vec<WorkspaceRoleRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRoleRepository::from_pool(pool).await?;
    let all_roles = repo.all().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(all_roles
        .into_iter()
        .filter(|r| r.workspace_id().to_string() == workspace_id)
        .map(|r| {
            WorkspaceRoleRow::new(
                r.id().clone(),
                r.workspace_id().clone(),
                r.name().map(ToOwned::to_owned),
            )
        })
        .collect())
}

/// Returns all workspace roles enriched with their permission IDs and names.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn list_roles_with_permissions(
    workspace_id: &str,
) -> Result<Vec<WorkspaceRoleWithPermissionsRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRoleRepository::from_pool(pool).await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    repo.find_with_permissions(&ws_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Records a `WorkspaceSettingsUpdated` event for the given workspace.
pub async fn update_workspace_settings(
    workspace_id: &str,
    name: Option<String>,
    timezone: String,
    date_format: String,
    currency: String,
    week_start: String,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;

    let agg_id: WorkspaceId = workspace_id.parse()?;
    let cmd = WorkspaceCommand::new(repo);
    cmd.update_settings(agg_id, name, timezone, date_format, currency, week_start)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Creates a new role in the given workspace.
///
/// # Errors
///
/// Returns an error if `name` is empty or the command fails.
pub async fn create_role(workspace_id: &str, name: String) -> Result<WorkspaceRoleId> {
    if name.trim().is_empty() {
        anyhow::bail!("role name must not be empty");
    }
    let pool = Pool::connect_admin().await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    let role_id = WorkspaceRoleId::new();
    let _ = WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool).await?)
        .create(role_id.clone(), ws_id, Some(name))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(role_id)
}

/// Renames an existing workspace role.
///
/// # Errors
///
/// Returns an error if `name` is empty or the command fails.
pub async fn rename_role(role_id: &str, name: String) -> Result<()> {
    if name.trim().is_empty() {
        anyhow::bail!("role name must not be empty");
    }
    let pool = Pool::connect_admin().await?;
    let id: WorkspaceRoleId = role_id.parse()?;
    WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool).await?)
        .rename(id, name)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Deletes a workspace role.
///
/// # Errors
///
/// Returns an error if any members still have this role assigned, or if the command fails.
pub async fn delete_role(role_id: &str) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let id: WorkspaceRoleId = role_id.parse()?;
    let repo = WorkspaceRoleRepository::from_pool(pool).await?;
    let count = repo
        .count_members_with_role(&id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if count > 0 {
        anyhow::bail!(
            "cannot delete role: {count} member(s) still assigned — reassign them first"
        );
    }
    WorkspaceRoleCommand::new(repo)
        .delete(id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Grants a permission to a workspace role.
pub async fn grant_role_permission(role_id: &str, permission_id: &str) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let id: WorkspaceRoleId = role_id.parse()?;
    let perm_id: PermissionId = permission_id.parse()?;
    WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool).await?)
        .grant_permission(id, perm_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Revokes a permission from a workspace role.
pub async fn revoke_role_permission(role_id: &str, permission_id: &str) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let id: WorkspaceRoleId = role_id.parse()?;
    let perm_id: PermissionId = permission_id.parse()?;
    WorkspaceRoleCommand::new(WorkspaceRoleRepository::from_pool(pool).await?)
        .revoke_permission(id, perm_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Returns all permissions available in the system.
pub async fn list_all_permissions() -> Result<Vec<PermissionRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = PermissionRepository::from_pool(pool).await?;
    let roots = repo.all().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(roots
        .into_iter()
        .map(|r| PermissionRow::new(r.id().clone(), r.name().to_string()))
        .collect())
}

/// Returns all members of the given workspace with their role and permission IDs.
pub async fn list_workspace_members(workspace_id: &str) -> Result<Vec<MemberRow>> {
    let pool = Pool::connect_admin().await?;
    let repo = WorkspaceRepository::from_pool(pool).await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    repo.find_members(&ws_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Assigns a role to a workspace member.
pub async fn assign_member_role(
    workspace_id: &str,
    user_id: &str,
    role_id: &str,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    let uid: UserId = user_id.parse()?;
    let rid: WorkspaceRoleId = role_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .assign_user_role(ws_id, uid, rid)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Revokes a role from a workspace member.
pub async fn revoke_member_role(
    workspace_id: &str,
    user_id: &str,
    role_id: &str,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    let uid: UserId = user_id.parse()?;
    let rid: WorkspaceRoleId = role_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .revoke_user_role(ws_id, uid, rid)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Grants a direct permission to a workspace member.
pub async fn grant_member_permission(
    workspace_id: &str,
    user_id: &str,
    permission_id: &str,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    let uid: UserId = user_id.parse()?;
    let perm_id: PermissionId = permission_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .grant_user_permission(ws_id, uid, perm_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Revokes a direct permission from a workspace member.
pub async fn revoke_member_permission(
    workspace_id: &str,
    user_id: &str,
    permission_id: &str,
) -> Result<()> {
    let pool = Pool::connect_admin().await?;
    let ws_id: WorkspaceId = workspace_id.parse()?;
    let uid: UserId = user_id.parse()?;
    let perm_id: PermissionId = permission_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .revoke_user_permission(ws_id, uid, perm_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Removes a member from a workspace, revoking all their roles and direct permissions.
///
/// # Errors
///
/// Returns an error if the user is the last admin of the workspace.
pub async fn remove_member(workspace_id: &str, user_id: &str) -> Result<()> {
    let pool = Pool::connect_admin().await?;

    // Guard: prevent removing the last admin.
    let admin_count: i64 = sqlx::query(
        "SELECT COUNT(DISTINCT wur.user_id) \
         FROM projections__workspace_user_roles wur \
         JOIN projections__workspace_roles wr ON wur.workspace_role_id = wr.id \
         WHERE wur.workspace_id = ? AND wr.name = 'admin'",
    )
    .bind(workspace_id)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?
    .try_get(0)
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    if admin_count <= 1 {
        let is_admin: i64 = sqlx::query(
            "SELECT COUNT(*) \
             FROM projections__workspace_user_roles wur \
             JOIN projections__workspace_roles wr ON wur.workspace_role_id = wr.id \
             WHERE wur.workspace_id = ? AND wur.user_id = ? AND wr.name = 'admin'",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .try_get(0)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        if is_admin > 0 {
            anyhow::bail!("cannot remove the last admin of a workspace");
        }
    }

    let ws_id: WorkspaceId = workspace_id.parse()?;
    let uid: UserId = user_id.parse()?;
    WorkspaceCommand::new(WorkspaceRepository::from_pool(pool).await?)
        .remove_member(ws_id, uid)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}
