use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::activity::domain::aggregates::Activity,
};

pub trait ActivityRepository<R>: Repository<Activity, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<Activity, R>>::Error>
        + From<<Self as WriteRepository<Activity>>::Error>;
}
