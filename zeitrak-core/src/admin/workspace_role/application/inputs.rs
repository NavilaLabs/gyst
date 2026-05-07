use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateWorkspaceRoleInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: Option<String>,
}
