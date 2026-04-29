use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root};

use crate::shared::AggregateId;

pub trait RecordToRow<R>
{
    type Row: Debug + Send + Sync;
    type Error: Debug + Send + Sync;

    /// # Errors
    ///
    /// Returns an error if the row cannot be converted to the view type.
    fn record_to_row(&self, record: R) -> Result<Self::Row, Self::Error>;
}

#[async_trait]
pub trait ReadRepository<T>
where
    T: Aggregate
{
    type Error: Debug + Send + Sync;
    /// The filter expression type used by `_by` methods.
    /// Each implementation binds this to its own query-builder type
    /// (e.g. `sea_query::Expr`), keeping this trait backend-agnostic.
    type Filter: Send + Sync + 'static;

    async fn get(&self, id: AggregateId) -> Result<Root<T>, Self::Error>;

    /// Returns the record wrapped in `Some`, or `None` if it does not exist.
    async fn find(&self, id: AggregateId) -> Result<Option<Root<T>>, Self::Error>;

    /// Returns `true` if a record with the given id exists.
    async fn exists(&self, id: AggregateId) -> Result<bool, Self::Error> {
        self.find(id).await.map(|opt| opt.is_some())
    }

    /// Returns the first record matching `filter`, or `None` if none match.
    async fn find_by(&self, filter: Self::Filter) -> Result<Option<Root<T>>, Self::Error>;

    /// Returns `true` if any record matches `filter`.
    async fn exists_by(&self, filter: Self::Filter) -> Result<bool, Self::Error> {
        self.find_by(filter).await.map(|opt| opt.is_some())
    }

    /// Returns all records whose id is in `ids`, silently omitting missing ones.
    async fn find_many(&self, ids: Vec<AggregateId>) -> Result<Vec<Root<T>>, Self::Error>;

    /// Returns `true` if every id in `ids` has a corresponding record.
    async fn exists_many(&self, ids: Vec<AggregateId>) -> Result<bool, Self::Error> {
        let expected = ids.len();
        self.find_many(ids)
            .await
            .map(|found| found.len() == expected)
    }

    /// Returns all records matching `filter`.
    async fn find_many_by(&self, filter: Self::Filter) -> Result<Vec<Root<T>>, Self::Error>;

    /// Returns `true` if any record matches `filter`.
    async fn exists_many_by(&self, filter: Self::Filter) -> Result<bool, Self::Error> {
        self.find_many_by(filter)
            .await
            .map(|found| !found.is_empty())
    }

    /// Returns the number of records whose id is in `ids`.
    async fn count_many(&self, ids: Vec<AggregateId>) -> Result<u64, Self::Error> {
        self.find_many(ids).await.map(|found| found.len() as u64)
    }

    /// Returns the number of records matching `filter`.
    async fn count_by(&self, filter: Self::Filter) -> Result<u64, Self::Error>;

    /// Returns all records.
    async fn all(&self) -> Result<Vec<Root<T>>, Self::Error>;

    /// Returns the total number of records.
    async fn count(&self) -> Result<u64, Self::Error>;
}

#[async_trait]
pub trait WriteRepository<T>
where
    T: Aggregate,
{
    type Error: Debug + Send + Sync;

    async fn save(&self, root: &mut Root<T>) -> Result<(), Self::Error>;
}
