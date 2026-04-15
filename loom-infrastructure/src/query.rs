use std::fmt::Debug;

use async_trait::async_trait;
use uuid::Uuid;

pub trait RowToView<R> {
    type View;
    type Error: Debug;

    /// # Errors
    ///
    /// Returns an error if the row cannot be converted to the view type.
    fn row_to_view(&self, row: R) -> Result<Self::View, Self::Error>;
}

#[async_trait]
pub trait Query<R>: RowToView<R> {
    /// The filter expression type used by `_by` methods.
    /// Each implementation binds this to its own query-builder type
    /// (e.g. `sea_query::Expr`), keeping this trait backend-agnostic.
    type Filter: Send + 'static;

    /// Returns the record, or an error if it does not exist.
    async fn une(&self, id: Uuid) -> Result<Self::View, Self::Error>;

    /// Returns the record wrapped in `Some`, or `None` if it does not exist.
    async fn find_one(&self, id: Uuid) -> Result<Option<Self::View>, Self::Error>;

    /// Returns `true` if a record with the given id exists.
    async fn exists_one(&self, id: Uuid) -> Result<bool, Self::Error> {
        self.find_one(id).await.map(|opt| opt.is_some())
    }

    /// Returns the first record matching `filter`, or an error if none match.
    async fn une_by(&self, filter: Self::Filter) -> Result<Self::View, Self::Error>;

    /// Returns the first record matching `filter`, or `None` if none match.
    async fn find_one_by(&self, filter: Self::Filter) -> Result<Option<Self::View>, Self::Error>;

    /// Returns `true` if any record matches `filter`.
    async fn exists_one_by(&self, filter: Self::Filter) -> Result<bool, Self::Error> {
        self.find_one_by(filter).await.map(|opt| opt.is_some())
    }

    /// Returns all records.
    async fn all(&self) -> Result<Vec<Self::View>, Self::Error>;

    /// Returns all records whose id is in `ids`, silently omitting missing ones.
    async fn find_many(&self, ids: Vec<Uuid>) -> Result<Vec<Self::View>, Self::Error>;

    /// Returns `true` if every id in `ids` has a corresponding record.
    async fn exists_many(&self, ids: Vec<Uuid>) -> Result<bool, Self::Error> {
        let expected = ids.len();
        self.find_many(ids)
            .await
            .map(|found| found.len() == expected)
    }

    /// Returns the number of records whose id is in `ids`.
    async fn count_many(&self, ids: Vec<Uuid>) -> Result<u64, Self::Error> {
        self.find_many(ids).await.map(|found| found.len() as u64)
    }

    /// Returns all records matching `filter`.
    async fn find_many_by(&self, filter: Self::Filter) -> Result<Vec<Self::View>, Self::Error>;

    /// Returns `true` if any record matches `filter`.
    async fn exists_many_by(&self, filter: Self::Filter) -> Result<bool, Self::Error> {
        self.find_many_by(filter)
            .await
            .map(|found| !found.is_empty())
    }

    /// Returns the number of records matching `filter`.
    async fn count_by(&self, filter: Self::Filter) -> Result<u64, Self::Error>;

    /// Returns the total number of records.
    async fn count(&self) -> Result<u64, Self::Error>;
}
