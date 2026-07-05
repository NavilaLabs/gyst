pub mod admin;
pub mod plugin;
pub mod tenant;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateId(pub Uuid);
