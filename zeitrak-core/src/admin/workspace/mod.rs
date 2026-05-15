pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    WorkspaceRoot,
    commands::{WorkspaceCommand, WorkspaceCommandTrait},
    inputs::CreateWorkspaceInput,
    queries::{WorkspaceQuery, WorkspaceQueryTrait},
    rows::{MemberRow, WorkspaceRow},
};
pub use domain::{
    aggregates::{Workspace, WorkspaceId},
    events::WorkspaceEvent,
    interfaces::WorkspaceRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace already exists")]
    AlreadyExists,
    #[error("workspace not found")]
    NotFound,
}
