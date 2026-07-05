pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    PermissionRoot,
    commands::{PermissionCommand, PermissionCommandTrait},
    inputs::CreatePermissionInput,
    queries::{PermissionQuery, PermissionQueryTrait},
    rows::PermissionRow,
};
pub use domain::{
    aggregates::{Permission, PermissionId},
    events::PermissionEvent,
    interfaces::PermissionRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("permission already exists")]
    AlreadyExists,
    #[error("permission not found")]
    NotFound,
}
