use validator::Validate;

#[derive(Clone, Validate)]
pub struct CreateInvitationInput {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(range(min = 1, max = 30, message = "TTL must be between 1 and 30 days"))]
    pub ttl_days: u32,
}
