pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    PermissionRoot,
    commands::{PermissionCommand, PermissionCommandTrait},
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
}
