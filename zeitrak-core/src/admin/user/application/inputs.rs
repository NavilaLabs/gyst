use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateUserInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
}

#[derive(Clone, Validate)]
pub struct UpdateUserSettingsInput {
    #[validate(length(min = 1, max = 100, message = "Timezone must not be empty"))]
    pub timezone: String,
    pub date_format: String,
    pub language: String,
}
