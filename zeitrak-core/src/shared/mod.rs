pub mod event_bus;
pub mod repositories;

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateId(pub Uuid);

impl AggregateId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for AggregateId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for AggregateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AggregateId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for AggregateId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_non_nil_uuid() {
        assert!(!AggregateId::new().0.is_nil());
    }

    #[test]
    fn two_new_ids_are_different() {
        assert_ne!(AggregateId::new(), AggregateId::new());
    }

    #[test]
    fn default_is_non_nil() {
        assert!(!AggregateId::default().0.is_nil());
    }

    #[test]
    fn from_str_parses_valid_uuid() {
        let s = "019d0ce8-facb-7c90-b9d7-287ae4f17c91";
        let id: AggregateId = s.parse().expect("must parse");
        assert_eq!(id.to_string(), s);
    }

    #[test]
    fn from_str_rejects_invalid_input() {
        assert!("not-a-uuid".parse::<AggregateId>().is_err());
    }

    #[test]
    fn display_roundtrips_through_from_str() {
        let id = AggregateId::new();
        let id2: AggregateId = id.to_string().parse().expect("must parse");
        assert_eq!(id, id2);
    }

    #[test]
    fn from_uuid_preserves_inner_value() {
        let uuid = Uuid::parse_str("019d0ce8-facb-7c90-b9d7-287ae4f17c91").unwrap();
        let id = AggregateId::from(uuid);
        assert_eq!(id.0, uuid);
    }
}
