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
| Command struct | `{PASCAL}Command<R>` — generic over `R: {PASCAL}Repository` | `{PASCAL}Command` — no repository, decorated with macro |
| `aggregate_root` macro target | Separate `{PASCAL}Root` zero-field struct | Applied directly to `{PASCAL}Command` |
| Command methods | `async fn` with `async_trait` | Synchronous `fn` |
| Command error type | `crate::Error<R, {PASCAL}>` | `{MODULE}::Error` (local) |
| `application/mod.rs` | Declares `{PASCAL}Root` struct | Re-exports `{PASCAL}Command as {PASCAL}Root` |
| Has `queries.rs` | Yes | No |
| Read model filename | `rows.rs` | `views.rs` |
| `interfaces.rs` style | `async_trait`, `From<{MODULE}::Error>` bound, `in_memory_repository` test mod | Simple trait, no async, no From<Error> bound, no test mod |
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
    commands::{PASCAL_CMDCommand, PASCAL_CMDCommandTrait},
    queries::{PASCAL_QUERYQuery, PASCAL_QUERY_TRAITQueryTrait},
    rows::PASCAL_ROWRow,
};
pub use domain::{
    aggregates::{PASCALStruct, PASCAL_IDId},
    events::PASCAL_EVENTEvent,
    interfaces::PASCAL_REPORepository,
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
    aggregates:{{PASCAL}, {PASCAL}Id, Error},
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

    #[test]
    fn apply_created_to_no_state_builds_{MODULE}() {
        let id = test_id();
        let event = {PASCAL}Event::Created { id: id.clone(), name: "Test".to_string() };
        let result = {PASCAL}::apply(None, event);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &id);
    }

    #[test]
    fn apply_created_to_existing_returns_already_exists() {
        let id = test_id();
        let existing = {PASCAL}::apply(
            None,
            {PASCAL}Event::Created { id: id.clone(), name: "First".to_string() },
        ).unwrap();
        let result = {PASCAL}::apply(
            Some(existing),
            {PASCAL}Event::Created { id, name: "Second".to_string() },
        );
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

    #[test]
    fn apply_created_to_no_state_builds_{MODULE}() {
        let id = test_id();
        let event = {PASCAL}Event::Created { id: id.clone(), name: "Test".to_string() };
        let result = {PASCAL}::apply(None, event);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &id);
    }

    #[test]
    fn apply_created_to_existing_returns_already_exists() {
        let id = test_id();
        let existing = {PASCAL}::apply(
            None,
            {PASCAL}Event::Created { id: id.clone(), name: "First".to_string() },
        ).unwrap();
        let result = {PASCAL}::apply(
            Some(existing),
            {PASCAL}Event::Created { id, name: "Second".to_string() },
        );
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

**Admin** (async, with `From<{MODULE}::Error>` bound, with test double):
```rust
use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    admin::{MODULE}::{self, domain::aggregates::{PASCAL}},
    shared::repositories::{ReadRepository, WriteRepository},
};

#[async_trait]
pub trait {PASCAL}Repository: ReadRepository<{PASCAL}> + WriteRepository<{PASCAL}> + Send + Sync {
    type Error: Debug
        + Send
        + Sync
        + From<{MODULE}::Error>
        + From<<Self as ReadRepository<{PASCAL}>>::Error>
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
        shared::{AggregateId, repositories::{ReadRepository, WriteRepository}},
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

    // {MODULE} is in scope via `use super::*` which pulls in the parent's
    // `use crate::admin::{MODULE}::{self, ...}` import.
    impl From<{MODULE}::Error> for StubError {
        fn from(_: {MODULE}::Error) -> Self { Self }
    }

    #[derive(Debug)]
    pub struct InMemory{PASCAL}Repository;

    impl InMemory{PASCAL}Repository {
        pub fn new() -> Self { Self }
    }

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
    impl ReadRepository<{PASCAL}> for InMemory{PASCAL}Repository {
        type Error = StubError;
        type Filter = ();

        async fn find(&self, _id: AggregateId) -> Result<Option<Root<{PASCAL}>>, Self::Error> { Ok(None) }
        async fn find_by(&self, _filter: ()) -> Result<Option<Root<{PASCAL}>>, Self::Error> { Ok(None) }
        async fn find_many(&self, _ids: Vec<AggregateId>) -> Result<Vec<Root<{PASCAL}>>, Self::Error> { Ok(vec![]) }
        async fn find_many_by(&self, _filter: ()) -> Result<Vec<Root<{PASCAL}>>, Self::Error> { Ok(vec![]) }
        async fn all(&self) -> Result<Vec<Root<{PASCAL}>>, Self::Error> { Ok(vec![]) }
        async fn count_by(&self, _filter: ()) -> Result<u64, Self::Error> { Ok(0) }
        async fn count(&self) -> Result<u64, Self::Error> { Ok(0) }
    }

    #[async_trait]
    impl WriteRepository<{PASCAL}> for InMemory{PASCAL}Repository {
        type Error = StubError;
    }

    #[async_trait]
    impl {PASCAL}Repository for InMemory{PASCAL}Repository {
        type Error = StubError;
    }
}
```

**Tenant** (simple, no async, no test double):
```rust
use std::fmt::Debug;

use crate::{
    shared::repositories::{ReadRepository, WriteRepository},
    tenant::{MODULE}::domain::aggregates::{PASCAL},
};

pub trait {PASCAL}Repository:
    ReadRepository<{PASCAL}> + WriteRepository<{PASCAL}> + Send + Sync
{
    type Error: Debug
        + Send
        + Sync
        + From<<Self as ReadRepository<{PASCAL}>>::Error>
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

**Admin** (async, repository-injected):
```rust
use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::{Aggregate, Root};

use crate::admin::{MODULE}::{
    application::{PASCAL}Root,
    domain::{
        aggregates::{PASCAL}, {PASCAL}Id},
        events::{PASCAL}Event,
        interfaces::{PASCAL}Repository,
    },
};

#[async_trait]
pub trait {PASCAL}CommandTrait<T>
where
    T: Aggregate,
{
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<Root<T>, Self::Error>;
}

#[derive(Debug)]
pub struct {PASCAL}Command<R> {
    repository: R,
}

impl<R> {PASCAL}Command<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> {PASCAL}CommandTrait<{PASCAL}> for {PASCAL}Command<R>
where
    R: Debug + {PASCAL}Repository,
{
    type Error = crate::Error<R, {PASCAL}>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    async fn create(
        &self,
        id: {PASCAL}Id,
        name: String,
    ) -> Result<Root<{PASCAL}>, <Self as {PASCAL}CommandTrait<{PASCAL}>>::Error> {
        Ok(Root::<{PASCAL}>::record_new(
            {PASCAL}Event::Created { id, name }.into(),
        )?)
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
    }
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

The query trait starts empty — add methods as real query needs emerge.

```rust
use std::fmt::Debug;

use crate::admin::{MODULE}::domain::interfaces::{PASCAL}Repository;

pub trait {PASCAL}QueryTrait {
    type Error: Debug + Send + Sync;
}

#[derive(Debug, Clone)]
pub struct {PASCAL}Query<R> {
    repository: R,
}

impl<R> {PASCAL}Query<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> {PASCAL}QueryTrait for {PASCAL}Query<R>
where
    R: Debug + {PASCAL}Repository,
{
    type Error = <R as {PASCAL}Repository>::Error;
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
impl<Repo, Agg> From<{MODULE}::Error> for crate::Error<Repo, Agg>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg> + WriteRepository<Agg>,
{
    fn from(value: {MODULE}::Error) -> Self {
        Self::AdminError(Error::{PASCAL}Error(value))
    }
}
```

**Tenant** (routes through `TenantError`):
```rust
impl<Repo, Agg> From<{MODULE}::Error> for crate::Error<Repo, Agg>
where
    Agg: Debug + Aggregate,
    Repo: ReadRepository<Agg> + WriteRepository<Agg>,
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

1. **Wrong error type in admin `interfaces.rs`**: The `Error` associated type in admin `{PASCAL}Repository` must include `From<{MODULE}::Error>` — tenant repositories do NOT have this bound.

2. **Wrong file name for read model**: Admin uses `rows.rs`, tenant uses `views.rs`. Both define a struct named `{PASCAL}Row`.

3. **Async in tenant commands**: Tenant command methods are synchronous — do NOT add `async_trait` or `async fn` to tenant `{PASCAL}CommandTrait` or `impl {PASCAL}Command`.

4. **Forgetting `From` impl in scope `mod.rs`**: Adding only the Error variant without the explicit `From<{MODULE}::Error> for crate::Error<Repo, Agg>` impl will break command handlers that use `?` on domain errors.

5. **Wrong `Error` type in tenant `Aggregate::apply`**: Tenant uses `type Error = Error` (the local `Error` from `aggregate_errors!`), not `{MODULE}::Error` — those are the same thing, but the import path differs.

6. **`application/mod.rs` for admin needs the `use` import**: The `#[eventually_macros::aggregate_root({PASCAL})]` macro needs the aggregate type to be in scope — add `use crate::admin::{MODULE}::{PASCAL};` at the top.

7. **Tenant `mod.rs` re-exports `Error` from aggregates**: `pub use domain::aggregates::{..., Error}` — the `Error` must be in this re-export list, not defined separately in `mod.rs`.

8. **`in_memory_repository` import path**: In the test module inside `interfaces.rs`, import `crate::admin::{MODULE}::{MODULE}::Error as {PASCAL}Error` is wrong — use `crate::admin::{MODULE}::Error` directly (it's already in scope via the parent use).
