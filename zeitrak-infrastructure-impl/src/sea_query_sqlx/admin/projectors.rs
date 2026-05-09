use async_trait::async_trait;
use eventually_projection::{Projector, RawEvent};

use crate::{
    Pool, ScopeAdmin, StateConnected,
    sea_query_sqlx::admin::{
        permission::projectors::PermissionProjector, user::projectors::UserProjector,
        workspace::projectors::WorkspaceProjector,
        workspace_role::projectors::WorkspaceRoleProjector,
    },
};

/// A single projector that dispatches each event to all admin sub-projectors
/// in a fixed, deterministic order.
///
/// Running all projectors under one [`ProjectionRunner`] with one shared
/// checkpoint guarantees that events are applied sequentially across every
/// projection table.  This prevents FK race conditions that arise when
/// projectors run in independent parallel loops (e.g. `WorkspaceRoleCreated`
/// being applied before the corresponding `WorkspaceCreated` has been
/// committed to `projections__workspaces`).
pub struct AdminProjector {
    user: UserProjector,
    workspace: WorkspaceProjector,
    workspace_role: WorkspaceRoleProjector,
    permission: PermissionProjector,
}

impl AdminProjector {
    #[must_use]
    pub fn new(pool: Pool<ScopeAdmin, StateConnected>) -> Self {
        Self {
            user: UserProjector::new(pool.clone()),
            workspace: WorkspaceProjector::new(pool.clone()),
            workspace_role: WorkspaceRoleProjector::new(pool.clone()),
            permission: PermissionProjector::new(pool),
        }
    }
}

#[async_trait]
impl Projector for AdminProjector {
    type Error = crate::Error;

    async fn handle(&mut self, event: RawEvent) -> Result<(), Self::Error> {
        self.user.handle(event.clone()).await?;
        self.workspace.handle(event.clone()).await?;
        self.workspace_role.handle(event.clone()).await?;
        self.permission.handle(event).await?;
        Ok(())
    }
}
