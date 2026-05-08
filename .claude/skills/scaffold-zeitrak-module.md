# Skill: scaffold-zeitrak-module

## Usage

```
/scaffold-zeitrak-module <scope>/<module_name>
```

- `scope` must be `admin` or `tenant`
- `module_name` must be snake_case (e.g., `activity_type`, `workspace_role`)

**Examples:**
- `/scaffold-zeitrak-module admin/api_key`
- `/scaffold-zeitrak-module tenant/time_entry`

---

## Step 1 — Parse arguments and derive names

Arguments are provided as: `$ARGUMENTS`

Extract `SCOPE` and `MODULE` from the argument (format: `scope/module_name`).

Then derive all naming variants:

| Variable   | Rule                                      | Example (input: `time_entry`)  |
|------------|-------------------------------------------|-------------------------------|
| `MODULE`   | raw snake_case input                      | `time_entry`                  |
| `PASCAL`   | split on `_`, capitalise each, join       | `TimeEntry`                   |
| `PASCAL_ID`| `{PASCAL}Id`                              | `TimeEntryId`                 |
| `PASCAL_EVENT` | `{PASCAL}Event`                       | `TimeEntryEvent`              |
| `PASCAL_REPO`  | `{PASCAL}Repository`                  | `TimeEntryRepository`         |
| `PASCAL_CMD`   | `{PASCAL}Command`                     | `TimeEntryCommand`            |
| `PASCAL_CMD_TRAIT` | `{PASCAL}CommandTrait`            | `TimeEntryCommandTrait`       |
| `PASCAL_ROOT`  | `{PASCAL}Root` (**admin only**)       | `TimeEntryRoot`               |
| `PASCAL_QUERY` | `{PASCAL}Query` (**admin only**)      | `TimeEntryQuery`              |
| `PASCAL_QUERY_TRAIT` | `{PASCAL}QueryTrait` (**admin only**) | `TimeEntryQueryTrait` |
| `PASCAL_ROW`   | `{PASCAL}Row`                         | `TimeEntryRow`                |
| `TYPE_NAME`    | raw snake_case (same as `MODULE`)     | `time_entry`                  |

---

## Step 2 — Pattern differences: admin vs tenant

| Aspect | Admin pattern | Tenant pattern |
|--------|--------------|----------------|
| `Error` enum location | `{MODULE}/mod.rs` (thiserror) | `domain/aggregates.rs` (via `crate::aggregate_errors!`) |
| Repository trait | `{PASCAL}Repository<R>` generic over row type `R`, extends `Repository<{PASCAL}, R>` | `{PASCAL}Repository<R>` generic over row type `R`, extends `Repository<{PASCAL}, R>` |
| Command trait | `{PASCAL}CommandTrait<R>` — generic over row type `R` | `{PASCAL}CommandTrait<T>` — no repository, decorated with macro |
| Command impl | `impl<Repo, R> {PASCAL}CommandTrait<R> for {PASCAL}Command<Repo>` | Applied directly to `{PASCAL}Command` struct via macro |
| Command methods | `async fn`, saves to repository via `self.repository.save(...)` | Synchronous `fn`, returns the decorated root directly |
| Command error type | `crate::Error<Repo, {PASCAL}, R>` | `{MODULE}::Error` (local) |
| `application/mod.rs` | Declares `{PASCAL}Root` struct with `aggregate_root` macro | Re-exports `{PASCAL}Command as {PASCAL}Root` |
| Has `queries.rs` | Yes — also generic over `R` | No |
| Read model filename | `rows.rs` | `views.rs` |
| `interfaces.rs` style | `async_trait`, `Repository<{PASCAL}, R>` supertrait, `From<{MODULE}::Error>` bound, `in_memory_repository` test mod | `Repository<{PASCAL}, R>` supertrait, no async methods, no `From<{MODULE}::Error>` bound, no test mod |
| Unit tests | `#[tokio::test]` async tests | `#[test]` sync tests |

---

## Step 3 — Create all files

The base path is `zeitrak-core/src/{SCOPE}/{MODULE}/`.

### 3.1 — `mod.rs` (top-level)

**Admin:**
```rust
pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    {PASCAL}Root,
    commands::{PASCAL}Command, {PASCAL}CommandTrait},
    queries::{PASCAL}Query, {PASCAL}QueryTrait},
    rows::{PASCAL}Row,
};
pub use domain::{
    aggregates::{PASCAL}, {PASCAL}Id},
    events::{PASCAL}Event,
    interfaces::{PASCAL}Repository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("already exists")]
    AlreadyExists,
}
```

Replace capitalised placeholders — example for `MODULE=api_key`, `PASCAL=ApiKey`:
```rust
pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    ApiKeyRoot,
    commands::{ApiKeyCommand, ApiKeyCommandTrait},
    queries::{ApiKeyQuery, ApiKeyQueryTrait},
    rows::ApiKeyRow,
};
pub use domain::{
    aggregates::{ApiKey, ApiKeyId},
    events::ApiKeyEvent,
    interfaces::ApiKeyRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("already exists")]
    AlreadyExists,
}
```

**Tenant:**
```rust
pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    {PASCAL}Root,
    commands::{PASCAL}Command, {PASCAL}CommandTrait},
    inputs::Create{PASCAL}Input,
    views::{PASCAL}Row,
};
pub use domain::{
    aggregates::{PASCAL}, {PASCAL}Id, Error},
    events::{PASCAL}Event,
    interfaces::{PASCAL}Repository,
};
```

---

### 3.2 — `domain/mod.rs`

Identical for both scopes:
```rust
pub mod aggregates;
pub mod events;
pub mod interfaces;
```

---

### 3.3 — `domain/aggregates.rs`

**Admin:**
```rust
use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::{
    admin::{MODULE}::{self, {PASCAL}Event},
    shared::AggregateId,
};

pub type {PASCAL}Id = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct {PASCAL} {
    id: {PASCAL}Id,
    name: String,
}

impl {PASCAL} {
    #[must_use]
    pub const fn id(&self) -> &{PASCAL}Id {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Aggregate for {PASCAL} {
    type Id = {PASCAL}Id;
    type Event = {PASCAL}Event;
    type Error = {MODULE}::Error;

    fn type_name() -> &'static str {
        "{TYPE_NAME}"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        // Start with just Created; add arms as you add event variants.
        // If you have mutation events, add (None, _) => Err({MODULE}::Error::NotFound) before them.
        match (state, event) {
            (None, {PASCAL}Event::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), {PASCAL}Event::Created { .. }) => Err({MODULE}::Error::AlreadyExists),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::{MODULE};

    fn test_id() -> {PASCAL}Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created_event(id: {PASCAL}Id, name: &str) -> {PASCAL}Event {
        {PASCAL}Event::Created { id, name: name.to_string() }
    }

    #[test]
    fn apply_created_to_no_state_builds_{MODULE}() {
        let id = test_id();
        let result = {PASCAL}::apply(None, created_event(id.clone(), "Test"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &id);
    }

    #[test]
    fn apply_created_to_existing_returns_already_exists() {
        let id = test_id();
        let existing = {PASCAL}::apply(None, created_event(id.clone(), "First")).unwrap();
        let result = {PASCAL}::apply(Some(existing), created_event(id, "Second"));
        assert!(matches!(result, Err({MODULE}::Error::AlreadyExists)));
    }
}
```

**Tenant:**
```rust
use eventually::aggregate::Aggregate;
use serde::{Deserialize, Serialize};

use crate::shared::AggregateId;
use crate::tenant::{MODULE}::{PASCAL}Event;

pub type {PASCAL}Id = AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct {PASCAL} {
    id: {PASCAL}Id,
    name: String,
}

impl {PASCAL} {
    #[must_use]
    pub const fn id(&self) -> &{PASCAL}Id {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

crate::aggregate_errors!("{TYPE_NAME}");

impl Aggregate for {PASCAL} {
    type Id = {PASCAL}Id;
    type Event = {PASCAL}Event;
    type Error = Error;

    fn type_name() -> &'static str {
        "{TYPE_NAME}"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        // Start with just Created; add arms as you add event variants.
        // If you have mutation events, add (None, _) => Err(Error::NotFound) before them.
        match (state, event) {
            (None, {PASCAL}Event::Created { id, name }) => Ok(Self { id, name }),
            (Some(_), {PASCAL}Event::Created { .. }) => Err(Error::AlreadyExists),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> {PASCAL}Id {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    fn created_event(id: {PASCAL}Id, name: &str) -> {PASCAL}Event {
        {PASCAL}Event::Created { id, name: name.to_string() }
    }

    #[test]
    fn apply_created_to_no_state_builds_{MODULE}() {
        let id = test_id();
        let result = {PASCAL}::apply(None, created_event(id.clone(), "Test"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &id);
    }

    #[test]
    fn apply_created_to_existing_returns_already_exists() {
        let id = test_id();
        let existing = {PASCAL}::apply(None, created_event(id.clone(), "First")).unwrap();
        let result = {PASCAL}::apply(Some(existing), created_event(id, "Second"));
        assert!(matches!(result, Err(Error::AlreadyExists)));
    }
}
```

---

### 3.4 — `domain/events.rs`

**Admin:**
```rust
use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::admin::{MODULE}::{PASCAL}Id;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum {PASCAL}Event {
    Created {
        id: {PASCAL}Id,
        name: String,
    },
}

impl Message for {PASCAL}Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "{PASCAL}Created",
        }
    }
}
```

**Tenant:**
```rust
use eventually::message::Message;
use serde::{Deserialize, Serialize};

use crate::tenant::{MODULE}::{PASCAL}Id;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum {PASCAL}Event {
    Created {
        id: {PASCAL}Id,
        name: String,
    },
}

impl Message for {PASCAL}Event {
    fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "{PASCAL}Created",
        }
    }
}
```

---

### 3.5 — `domain/interfaces.rs`

**Admin** — repository trait is generic over row type `R`; extends `Repository<{PASCAL}, R>`:
```rust
use std::fmt::Debug;

use async_trait::async_trait;
use crate::{
    admin::{MODULE}::{self, domain::aggregates::{PASCAL}},
    shared::repositories::{ReadRepository, Repository, WriteRepository},
};

#[async_trait]
pub trait {PASCAL}Repository<R>: Repository<{PASCAL}, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<{MODULE}::Error>
        + From<<Self as ReadRepository<{PASCAL}, R>>::Error>
        + From<<Self as WriteRepository<{PASCAL}>>::Error>;
}

#[cfg(test)]
pub mod in_memory_repository {
    use async_trait::async_trait;
    use eventually::aggregate::{
        Root,
        repository::{GetError, Getter, SaveError, Saver},
    };

    use super::*;
    use crate::{
        admin::{MODULE}::{PASCAL}Id,
        shared::{AggregateId, repositories::{ReadRepository, Repository, RowToRoot, WriteRepository}},
    };

    #[derive(Debug, thiserror::Error)]
    #[error("stub")]
    pub struct StubError;

    impl From<GetError> for StubError {
        fn from(_: GetError) -> Self { Self }
    }

    impl From<SaveError> for StubError {
        fn from(_: SaveError) -> Self { Self }
    }

    impl From<{MODULE}::Error> for StubError {
        fn from(_: {MODULE}::Error) -> Self { Self }
    }

    #[derive(Debug)]
    pub struct InMemory{PASCAL}Repository;

    impl InMemory{PASCAL}Repository {
        pub fn new() -> Self { Self }
    }

    impl RowToRoot<(), {PASCAL}> for InMemory{PASCAL}Repository {
        type Error = StubError;
        fn row_to_root(&self, _row: ()) -> Result<Root<{PASCAL}>, Self::Error> {
            unimplemented!("test stub")
        }
    }

    impl Repository<{PASCAL}, ()> for InMemory{PASCAL}Repository {}

    #[async_trait]
    impl Getter<{PASCAL}> for InMemory{PASCAL}Repository {
        async fn get(&self, _id: &{PASCAL}Id) -> Result<Root<{PASCAL}>, GetError> {
            unimplemented!("test stub")
        }
    }

    #[async_trait]
    impl Saver<{PASCAL}> for InMemory{PASCAL}Repository {
        async fn save(&self, _root: &mut Root<{PASCAL}>) -> Result<(), SaveError> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadRepository<{PASCAL}, ()> for InMemory{PASCAL}Repository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<{PASCAL}>>, StubError> { Ok(None) }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<{PASCAL}>>, StubError> { Ok(None) }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<{PASCAL}>>, StubError> { Ok(vec![]) }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<{PASCAL}>>, StubError> { Ok(vec![]) }
        async fn all(&self) -> Result<Vec<Root<{PASCAL}>>, StubError> { Ok(vec![]) }
        async fn count_by(&self, _filter: ()) -> Result<u64, StubError> { Ok(0) }
        async fn count(&self) -> Result<u64, StubError> { Ok(0) }
    }

    #[async_trait]
    impl WriteRepository<{PASCAL}> for InMemory{PASCAL}Repository {
        type Error = StubError;
    }

    #[async_trait]
    impl {PASCAL}Repository<()> for InMemory{PASCAL}Repository {
        type Error = StubError;
    }
}
```

**Tenant** (generic over row type `R`, no extra async methods, no test double):
```rust
use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, Repository, WriteRepository},
    tenant::{MODULE}::domain::aggregates::{PASCAL},
};

pub trait {PASCAL}Repository<R>: Repository<{PASCAL}, R> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<{PASCAL}, R>>::Error>
        + From<<Self as WriteRepository<{PASCAL}>>::Error>;
}
```

---

### 3.6 — `application/mod.rs`

**Admin:**
```rust
use crate::admin::{MODULE}::{PASCAL};

pub mod commands;
pub mod inputs;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root({PASCAL})]
#[derive(Debug, Clone, PartialEq)]
pub struct {PASCAL}Root;
```

**Tenant:**
```rust
pub mod commands;
pub mod inputs;
pub mod views;

pub use commands::{PASCAL}Command as {PASCAL}Root;
```

---

### 3.7 — `application/commands.rs`

**Admin** — trait is generic over `R` (row type); impl uses `<Repo, R>`; `create` saves to repository:
```rust
use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::{MODULE}::{
    application::{PASCAL}Root,
    domain::{
        aggregates::{PASCAL}, {PASCAL}Id},
        events::{PASCAL}Event,
        interfaces::{PASCAL}Repository,
    },
};

#[async_trait]
pub trait {PASCAL}CommandTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<Root<{PASCAL}>, Self::Error>;
}

#[derive(Debug)]
pub struct {PASCAL}Command<Repo> {
    repository: Repo,
}

impl<Repo> {PASCAL}Command<Repo> {
    pub fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> {PASCAL}CommandTrait<R> for {PASCAL}Command<Repo>
where
    R: Debug,
    Repo: Debug + {PASCAL}Repository<R>,
{
    type Error = crate::Error<Repo, {PASCAL}, R>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied or the root cannot be saved.
    async fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<Root<{PASCAL}>, <Self as {PASCAL}CommandTrait<R>>::Error> {
        let mut root = Root::<{PASCAL}>::record_new(
            {PASCAL}Event::Created { id, name }.into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(root)
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::{MODULE}::domain::interfaces::in_memory_repository::InMemory{PASCAL}Repository;

    use super::*;

    #[tokio::test]
    async fn create_returns_root_with_applied_state() {
        let id: {PASCAL}Id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = {PASCAL}Command::new(InMemory{PASCAL}Repository::new())
            .create(id.clone(), "Test".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &id);
    }
}
```

**Mutation command pattern** (when you need get → record → save):
```rust
/// # Errors
///
/// Returns an error if the domain event cannot be applied to the aggregate.
async fn rename(
    &self,
    id: {PASCAL}Id,
    name: String,
) -> Result<(), <Self as {PASCAL}CommandTrait<R>>::Error> {
    let mut root: {PASCAL}Root = self
        .repository
        .get(&id)
        .await
        .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
        .into();
    root.record_that({PASCAL}Event::Renamed { name }.into())?;
    self.repository
        .save(&mut root)
        .await
        .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
}
```

**Tenant** (sync, macro-decorated, no repository):
```rust
use std::fmt::Debug;

use eventually::aggregate;

use crate::tenant::{MODULE}::{
    self,
    domain::{
        aggregates::{PASCAL}, {PASCAL}Id},
        events::{PASCAL}Event,
    },
};

pub trait {PASCAL}CommandTrait<T> {
    type Error: Debug + Sync + Send;

    fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<T, Self::Error>;
}

#[eventually_macros::aggregate_root({PASCAL})]
pub struct {PASCAL}Command;

impl {PASCAL}CommandTrait<{PASCAL}Command> for {PASCAL}Command {
    type Error = {MODULE}::Error;

    fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<{PASCAL}Command, Self::Error> {
        Ok(aggregate::Root::<{PASCAL}>::record_new(
            {PASCAL}Event::Created { id, name }.into(),
        )?
        .into())
    }
}

impl {PASCAL}Command {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        id: {PASCAL}Id,
        name: String,
    ) -> Result<Self, {MODULE}::Error> {
        Ok(aggregate::Root::<{PASCAL}>::record_new(
            {PASCAL}Event::Created { id, name }.into(),
        )?
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: {PASCAL}Id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = {PASCAL}Command::create(id.clone(), "Test".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().aggregate_id(), &id);
    }
}
```

---

### 3.8 — `application/queries.rs` (admin only — skip for tenant)

The query trait is generic over `R` and starts empty — add methods as real query needs emerge.

```rust
use std::fmt::Debug;

use crate::admin::{MODULE}::domain::interfaces::{PASCAL}Repository;

pub trait {PASCAL}QueryTrait<R> {
    type Error: Debug + Send + Sync;
}

#[derive(Debug, Clone)]
pub struct {PASCAL}Query<Repo> {
    repository: Repo,
}

impl<Repo> {PASCAL}Query<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

impl<Repo, R> {PASCAL}QueryTrait<R> for {PASCAL}Query<Repo>
where
    Repo: Debug + {PASCAL}Repository<R>,
{
    type Error = <Repo as {PASCAL}Repository<R>>::Error;
}
```

---

### 3.9 — `application/rows.rs` (admin) / `application/views.rs` (tenant)

Both scopes use the same struct name `{PASCAL}Row`. Admin puts it in `rows.rs`, tenant in `views.rs`.

```rust
use crate::{SCOPE}::{MODULE}::{PASCAL}Id;

#[derive(Debug, Clone)]
pub struct {PASCAL}Row {
    id: {PASCAL}Id,
    name: String,
}

impl {PASCAL}Row {
    #[must_use]
    pub const fn new(id: {PASCAL}Id, name: String) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub const fn id(&self) -> &{PASCAL}Id {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
```

---

### 3.10 — `application/inputs.rs`

Identical for both scopes:
```rust
use validator::Validate;

#[derive(Clone, Validate)]
pub struct Create{PASCAL}Input {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
}
```

---

## Step 4 — Update parent scope `mod.rs`

Read the current content of `zeitrak-core/src/{SCOPE}/mod.rs` first, then make three additions:

### 4a — Add module declaration (at the top with the other `pub mod` lines)
```rust
pub mod {MODULE};
```

### 4b — Add Error variant to the scope `Error` enum

```rust
#[error("{0:?}")]
{PASCAL}Error(#[from] {MODULE}::Error),
```

### 4c — Add `From` impl (after the existing impls)

**Admin** (routes through `AdminError`):
```rust
impl<Repo, Agg, R> From<{MODULE}::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: {MODULE}::Error) -> Self {
        Self::AdminError(Error::{PASCAL}Error(value))
    }
}
```

**Tenant** (routes through `TenantError`):
```rust
impl<Repo, Agg, R> From<{MODULE}::Error> for crate::Error<Repo, Agg, R>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg, R> + WriteRepository<Agg>,
{
    fn from(value: {MODULE}::Error) -> Self {
        Self::TenantError(Error::{PASCAL}Error(value))
    }
}
```

---

## Step 5 — Verify

Run:
```bash
cargo check -p zeitrak-core
```

Fix any compilation errors before reporting success.

---

## Common mistakes

1. **Wrong error type in admin `interfaces.rs`**: The `Error` associated type in admin `{PASCAL}Repository<R>` must include `From<{MODULE}::Error>` — tenant repositories do NOT have this bound.

2. **Wrong file name for read model**: Admin uses `rows.rs`, tenant uses `views.rs`. Both define a struct named `{PASCAL}Row`.

3. **Async in tenant commands**: Tenant command methods are synchronous — do NOT add `async_trait` or `async fn` to tenant `{PASCAL}CommandTrait` or `impl {PASCAL}Command`.

4. **Forgetting `From` impl in scope `mod.rs`**: Adding only the Error variant without the explicit `From<{MODULE}::Error> for crate::Error<Repo, Agg, R>` impl will break command handlers that use `?` on domain errors.

5. **Wrong `Error` type in tenant `Aggregate::apply`**: Tenant uses `type Error = Error` (the local `Error` from `aggregate_errors!`), not `{MODULE}::Error` — those are the same thing, but the import path differs.

6. **`application/mod.rs` for admin needs the `use` import**: The `#[eventually_macros::aggregate_root({PASCAL})]` macro needs the aggregate type to be in scope — add `use crate::admin::{MODULE}::{PASCAL};` at the top.

7. **Tenant `mod.rs` re-exports `Error` from aggregates**: `pub use domain::aggregates::{..., Error}` — the `Error` must be in this re-export list, not defined separately in `mod.rs`.

8. **Admin command trait generic is `R`, not `T: Aggregate`**: `{PASCAL}CommandTrait<R>` is generic over the database row type `R`, not over an aggregate `T`. The aggregate type is fixed as `{PASCAL}` throughout. Do not add `where T: Aggregate` bounds.

9. **Admin `create` must save to repository**: After `Root::record_new(...)`, call `self.repository.save(&mut root).await.map_err(|e| crate::Error::WriteRepositoryError(e.into()))?`. Do not skip the save — it's not optional.

10. **Mutation commands must get before recording**: For commands that mutate existing state, use `self.repository.get(&id).await.map_err(|e| crate::Error::ReadRepositoryError(e.into()))?.into()` to fetch the current `{PASCAL}Root`, then `root.record_that(event.into())?`, then save.

11. **Admin query error type**: `type Error = <Repo as {PASCAL}Repository<R>>::Error` — the error comes from the repository's associated type via the `R` generic, not directly from `Repo::Error`.
