use serde::{Deserialize, Serialize};

pub const AGGREGATE_TYPE: &str = "tenant";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Aggregate {
    name: String,
}
