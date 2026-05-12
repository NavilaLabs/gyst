use validator::{Validate, ValidationError};

#[derive(Clone, Validate)]
pub struct CreateUserInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_upper && has_lower && has_digit && has_special {
        Ok(())
    } else {
        let mut error = ValidationError::new("password_strength");
        error.message =
            Some("Must contain uppercase, lowercase, number, and special character".into());
        Err(error)
    }
}

#[derive(Clone, Validate)]
pub struct UpdateUserSettingsInput {
    #[validate(length(min = 1, max = 100, message = "Timezone must not be empty"))]
    pub timezone: String,
    pub date_format: String,
    pub language: String,
}
