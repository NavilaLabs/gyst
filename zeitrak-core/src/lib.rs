use std::fmt::Debug;

use eventually::aggregate::Aggregate;

use crate::shared::repositories::{ReadRepository, WriteRepository};

pub mod admin;
pub mod permissions;
pub mod plugin;
pub mod shared;
pub mod tenant;
pub mod validation;

/// Generate the standard `AlreadyExists` / `NotFound` error enum for an aggregate.
///
/// This avoids repeating the same six-line boilerplate in every aggregate module.
///
/// # Usage
///
/// ```rust,ignore
/// crate::aggregate_errors!("customer");
/// // Expands to:
/// // #[derive(Debug)]
/// // pub enum Error { AlreadyExists, NotFound }
/// // impl Display for Error { … "customer already exists" / "customer not found" }
/// // impl std::error::Error for Error {}
/// ```
#[macro_export]
macro_rules! aggregate_errors {
    ($entity:literal) => {
        #[derive(Debug)]
        pub enum Error {
            AlreadyExists,
            NotFound,
        }

        impl ::std::fmt::Display for Error {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::AlreadyExists => write!(f, concat!($entity, " already exists")),
                    Self::NotFound => write!(f, concat!($entity, " not found")),
                }
            }
        }

        impl ::std::error::Error for Error {}
    };
}

#[derive(thiserror::Error)]
pub enum Error<Repo, Agg, R = ()>
where
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
    Agg: Debug + Aggregate,
{
    #[error("{0:?}")]
    AdminError(#[from] admin::Error),
    #[error("{0:?}")]
    TenantError(#[from] tenant::Error),
    #[error("{0:?}")]
    ReadRepositoryError(<Repo as ReadRepository<Agg, R>>::Error),
    #[error("{0:?}")]
    WriteRepositoryError(<Repo as WriteRepository<Agg>>::Error),
    #[error("{0:?}")]
    ParseUuidError(#[from] uuid::Error),
    #[error("{0:?}")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl<Repo, Agg, R> std::fmt::Debug for Error<Repo, Agg, R>
where
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
    Agg: Debug + Aggregate,
    <Repo as ReadRepository<Agg, R>>::Error: Debug,
    <Repo as WriteRepository<Agg>>::Error: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdminError(e) => write!(f, "AdminError({e:?})"),
            Self::TenantError(e) => write!(f, "TenantError({e:?})"),
            Self::ReadRepositoryError(e) => write!(f, "ReadRepositoryError({e:?})"),
            Self::WriteRepositoryError(e) => write!(f, "WriteRepositoryError({e:?})"),
            Self::ParseUuidError(e) => write!(f, "ParseUuidError({e:?})"),
            Self::SerdeJsonError(e) => write!(f, "SerdeJsonError({e:?})"),
        }
    }
}
