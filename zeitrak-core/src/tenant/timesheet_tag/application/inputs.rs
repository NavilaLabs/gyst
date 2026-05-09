use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateTimesheetTagInput {
    #[validate(length(min = 1, max = 100, message = "Tag name must not be empty"))]
    pub name: String,
}

#[derive(Clone, Validate)]
pub struct RenameTimesheetTagInput {
    #[validate(length(min = 1, max = 100, message = "Tag name must not be empty"))]
    pub name: String,
}
