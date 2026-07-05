use validator::Validate;

fn validate_hex_color(color: &str) -> Result<(), validator::ValidationError> {
    if color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Ok(());
    }
    let mut e = validator::ValidationError::new("hex_color");
    e.message = Some("Color must be a 6-digit hex code like #3b82f6".into());
    Err(e)
}

#[derive(Clone, Validate)]
pub struct CreateActivityInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
    #[validate(custom(function = "validate_hex_color"))]
    pub color: String,
}

#[derive(Clone, Validate)]
pub struct UpdateActivityInput {
    #[validate(length(min = 1, max = 255, message = "Name must not be empty"))]
    pub name: String,
    #[validate(custom(function = "validate_hex_color"))]
    pub color: String,
}
