use std::fmt::Debug;

use crate::{admin::permission::domain::aggregates::Permission, shared::repositories::{ReadRepository, WriteRepository}};

pub trait PermissionRepository: ReadRepository<Permission> + WriteRepository<Permission> + Send + Sync {
    type Error: Debug + Sync + Send + From<<Self as ReadRepository<Permission>>::Error> + From<<Self as WriteRepository<Permission>>::Error>;
}
