use std::fmt::Debug;

use async_trait::async_trait;
use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::activity::{application::views::ActivityRow, domain::aggregates::Activity},
};

#[async_trait]
pub trait ActivityRepository<R>: Repository<Activity, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<Activity, R>>::Error>
        + From<<Self as WriteRepository<Activity>>::Error>;

    /// Returns all non-deleted activities ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn list_all(&self) -> Result<Vec<ActivityRow>, <Self as ActivityRepository<R>>::Error>;
}
