use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreatePermissionInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
}
