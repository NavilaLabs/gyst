pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    WorkspaceRoleRoot,
    commands::{WorkspaceRoleCommand, WorkspaceRoleCommandTrait},
    inputs::CreateWorkspaceRoleInput,
    queries::{WorkspaceRoleQuery, WorkspaceRoleQueryTrait},
    rows::WorkspaceRoleRow,
};
pub use domain::{
    aggregates::{WorkspaceRole, WorkspaceRoleId},
    events::WorkspaceRoleEvent,
    interfaces::WorkspaceRoleRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace role already exists")]
    AlreadyExists,
    #[error("workspace role not found")]
    NotFound,
}
