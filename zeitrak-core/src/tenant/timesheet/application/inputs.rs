use validator::Validate;

#[derive(Clone, Validate)]
pub struct StartTimesheetInput {
    #[validate(length(min = 1, message = "Start time must not be empty"))]
    pub start_time: String,
    #[validate(length(min = 1, message = "Timezone must not be empty"))]
    pub timezone: String,
}
