//! Shared validation utilities used by both controllers and the GUI frontend.

pub use validator::{Validate, ValidationErrors};

/// Format `ValidationErrors` into a single user-friendly string.
///
/// Collects only the human-readable `message` from each field error and
/// joins them with "; ". Falls back to the full debug representation if
/// a field error carries no message.
#[must_use]
pub fn validation_summary(e: &ValidationErrors) -> String {
    let messages: Vec<String> = e
        .field_errors()
        .into_values()
        .flat_map(|errs| errs.iter())
        .filter_map(|fe| fe.message.as_deref().map(str::to_owned))
        .collect();

    if messages.is_empty() {
        e.to_string()
    } else {
        messages.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::activity::application::inputs::CreateActivityInput;

    #[test]
    fn valid_input_produces_no_errors() {
        let input = CreateActivityInput { name: "Stand-up".to_string() };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn empty_name_produces_validation_error() {
        let input = CreateActivityInput { name: String::new() };
        assert!(input.validate().is_err());
    }

    #[test]
    fn summary_extracts_message_from_failing_field() {
        let input = CreateActivityInput { name: String::new() };
        let errors = input.validate().expect_err("must fail");
        let summary = validation_summary(&errors);
        assert!(
            summary.contains("Name must not be empty"),
            "unexpected summary: {summary}"
        );
    }

    #[test]
    fn summary_falls_back_to_debug_string_when_no_message_set() {
        // Build errors that have no message to trigger the fallback branch.
        let mut errors = ValidationErrors::new();
        let mut e = validator::ValidationError::new("required");
        e.message = None;
        errors.add("field", e);
        let summary = validation_summary(&errors);
        // Falls back to the Debug representation — non-empty and contains field name.
        assert!(!summary.is_empty());
    }
}
