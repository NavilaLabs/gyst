use std::fmt::Debug;

use crate::{
    admin::permission::{self, domain::aggregates::Permission},
    shared::repositories::{ReadRepository, WriteRepository},
};

pub trait PermissionRepository:
    ReadRepository<Permission> + WriteRepository<Permission> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<permission::Error>
        + From<<Self as ReadRepository<Permission>>::Error>
        + From<<Self as WriteRepository<Permission>>::Error>;
}
