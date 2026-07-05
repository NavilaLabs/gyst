use serde::{Deserialize, Serialize};

pub type Id = crate::AggregateId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    id: Id,
    name: String,
    email: String,
    password: String,
    is_verified: bool,
    is_instance_admin: bool,
    verification_token: Option<String>,
}

impl Aggregate {
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub const fn is_verified(&self) -> bool {
        self.is_verified
    }

    pub const fn is_instance_admin(&self) -> bool {
        self.is_instance_admin
    }

    pub fn verification_token(&self) -> Option<&str> {
        self.verification_token.as_deref()
    }
}
