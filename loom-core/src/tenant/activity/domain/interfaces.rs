use std::fmt::Debug;

use crate::{shared::repositories::{ReadRepository, WriteRepository}, tenant::activity::domain::aggregates::Activity};

pub trait ActivityRepository: ReadRepository<Activity> + WriteRepository<Activity> + Send + Sync {
    type Error: Debug + Send + Sync + From<<Self as ReadRepository<Activity>>::Error> + From<<Self as WriteRepository<Activity>>::Error>;
}
