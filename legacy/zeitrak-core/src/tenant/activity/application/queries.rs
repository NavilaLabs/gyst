use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::tenant::activity::{
    application::views::ActivityRow,
    domain::aggregates::{Activity, ActivityId},
    domain::interfaces::ActivityRepository,
};

#[async_trait]
pub trait ActivityQueryTrait<R> {
    type Error: Debug + Send + Sync;

    async fn find_by_id(&self, id: ActivityId) -> Result<Option<Root<Activity>>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<Root<Activity>>, Self::Error>;
    async fn list_all(&self) -> Result<Vec<ActivityRow>, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct ActivityQuery<Repo> {
    repository: Repo,
}

impl<Repo> ActivityQuery<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> ActivityQueryTrait<R> for ActivityQuery<Repo>
where
    Repo: Debug + Send + Sync + ActivityRepository<R>,
{
    type Error = <Repo as ActivityRepository<R>>::Error;

    async fn find_by_id(&self, id: ActivityId) -> Result<Option<Root<Activity>>, Self::Error> {
        self.repository.find(id).await.map_err(Into::into)
    }

    async fn find_all(&self) -> Result<Vec<Root<Activity>>, Self::Error> {
        self.repository.all().await.map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<ActivityRow>, Self::Error> {
        self.repository.list_all().await
    }
}
