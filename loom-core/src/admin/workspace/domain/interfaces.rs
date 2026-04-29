use std::fmt::Debug;

use crate::{admin::workspace::{domain::aggregates::Workspace}, shared::repositories::{ReadRepository, WriteRepository}};

pub trait WorkspaceRepository: ReadRepository<Workspace> + WriteRepository<Workspace> + Send + Sync {
    type Error: Debug + Sync + Send + From<<Self as ReadRepository<Workspace>>::Error> + From<<Self as WriteRepository<Workspace>>::Error>;
}
