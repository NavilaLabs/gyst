use std::fmt::Debug;

use crate::{
    admin::workspace_role::{self, domain::aggregates::WorkspaceRole},
    shared::repositories::{ReadRepository, WriteRepository},
};

pub trait WorkspaceRoleRepository:
    ReadRepository<WorkspaceRole> + WriteRepository<WorkspaceRole> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<workspace_role::Error>
        + From<<Self as ReadRepository<WorkspaceRole>>::Error>
        + From<<Self as WriteRepository<WorkspaceRole>>::Error>;
}
