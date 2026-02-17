use std::fmt::Display;

use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An identifier for an aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateId(Uuid);

impl AggregateId {
    /// Generates a new aggregate identifier.
    pub fn generate() -> Self {
        Self(Uuid::new_v7(uuid::Timestamp::now(uuid::ContextV7::new())))
    }
}

impl Display for AggregateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityToken(Vec<u8>);

impl EntityToken {
    /// Generates a new entity token.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);

        Self(bytes.to_vec())
    }
}

impl Default for EntityToken {
    fn default() -> Self {
        Self(vec![0u8; 16])
    }
}

impl AsRef<[u8]> for EntityToken {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Display for EntityToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
