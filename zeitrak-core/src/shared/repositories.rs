use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root, repository::{GetError, Getter, SaveError, Saver}};

use crate::shared::AggregateId;

#[async_trait]
pub trait Repository<T, R>: ReadRepository<T, R> + WriteRepository<T>
where
    T: Aggregate,
{}

pub trait RowToRoot<R, T>
where
    T: Aggregate,
{
    type Error: Debug + Send + Sync;

    /// # Errors
    ///
    /// Returns an error if the row cannot be converted to an aggregate root.
    fn row_to_root(&self, row: R) -> Result<Root<T>, Self::Error>;
}

#[async_trait]
pub trait ReadRepository<T, R>: Send + Sync + Getter<T> + RowToRoot<R, T>
where
    T: Aggregate,
{
    type Error: Debug + Send + Sync + From<GetError> + From<<Self as RowToRoot<R, T>>::Error>;
    /// The filter expression type used by `_by` methods.
    /// Each implementation binds this to its own query-builder type
    /// (e.g. `sea_query::Expr`), keeping this trait backend-agnostic.
    type Filter: Send + Sync + 'static;

    /// Returns the record wrapped in `Some`, or `None` if it does not exist.
    async fn find(&self, id: AggregateId) -> Result<Option<Root<T>>, <Self as ReadRepository<T, R>>::Error>;

    /// Returns `true` if a record with the given id exists.
    async fn exists(&self, id: AggregateId) -> Result<bool, <Self as ReadRepository<T, R>>::Error> {
        self.find(id).await.map(|opt| opt.is_some())
    }

    /// Returns the first record matching `filter`, or `None` if none match.
    async fn find_by(&self, filter: Self::Filter) -> Result<Option<Root<T>>, <Self as ReadRepository<T, R>>::Error>;

    /// Returns all records whose id is in `ids`, silently omitting missing ones.
    async fn find_many(&self, ids: Vec<AggregateId>) -> Result<Vec<Root<T>>, <Self as ReadRepository<T, R>>::Error>;

    /// Returns all records matching `filter`.
    async fn find_many_by(&self, filter: Self::Filter) -> Result<Vec<Root<T>>, <Self as ReadRepository<T, R>>::Error>;

    /// Returns `true` if any record matches `filter`.
    async fn exists_by(&self, filter: Self::Filter) -> Result<bool, <Self as ReadRepository<T, R>>::Error> {
        self.find_by(filter).await.map(|opt| opt.is_some())
    }

    /// Returns `true` if every id in `ids` has a corresponding record.
    async fn exists_many(&self, ids: Vec<AggregateId>) -> Result<bool, <Self as ReadRepository<T, R>>::Error> {
        let expected = ids.len();
        self.find_many(ids)
            .await
            .map(|found| found.len() == expected)
    }

    /// Returns `true` if any record matches `filter`.
    async fn exists_many_by(&self, filter: Self::Filter) -> Result<bool, <Self as ReadRepository<T, R>>::Error> {
        self.find_many_by(filter)
            .await
            .map(|found| !found.is_empty())
    }

    /// Returns the number of records whose id is in `ids`.
    async fn count_many(&self, ids: Vec<AggregateId>) -> Result<u64, <Self as ReadRepository<T, R>>::Error> {
        self.find_many(ids).await.map(|found| found.len() as u64)
    }

    /// Returns the number of records matching `filter`.
    async fn count_by(&self, filter: Self::Filter) -> Result<u64, <Self as ReadRepository<T, R>>::Error>;

    /// Returns all records.
    async fn all(&self) -> Result<Vec<Root<T>>, <Self as ReadRepository<T, R>>::Error>;

    /// Returns the total number of records.
    async fn count(&self) -> Result<u64, <Self as ReadRepository<T, R>>::Error>;
}

#[async_trait]
pub trait WriteRepository<T>: Send + Sync + Saver<T>
where
    T: Aggregate,
{
    type Error: Debug + Send + Sync + From<SaveError>;
}
