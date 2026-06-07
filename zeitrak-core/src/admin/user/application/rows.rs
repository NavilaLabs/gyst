use crate::admin::user::UserId;

#[derive(Debug, Clone)]
pub struct UserRow {
    id: UserId,
    name: String,
    email: String,
    pub timezone: String,
    pub date_format: String,
    pub language: String,
    pub is_verified: bool,
    pub is_instance_admin: bool,
}

impl UserRow {
    #[must_use]
    pub fn new(id: UserId, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            timezone: "Europe/Berlin".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            language: "en".to_string(),
            is_verified: false,
            is_instance_admin: false,
        }
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new_with_settings(
        id: UserId,
        name: String,
        email: String,
        timezone: String,
        date_format: String,
        language: String,
        is_verified: bool,
        is_instance_admin: bool,
    ) -> Self {
        Self {
            id,
            name,
            email,
            timezone,
            date_format,
            language,
            is_verified,
            is_instance_admin,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &UserId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
