use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateActivityInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
}

#[derive(Clone, Validate)]
pub struct UpdateActivityInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
}
