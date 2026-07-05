use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateWorkspaceInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: Option<String>,
}
