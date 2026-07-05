mod component;
pub use component::DefaultLayout;

/// The accent color for the currently active workspace.
///
/// Provided by [`DefaultLayout`] and accessible anywhere in the subtree via
/// `use_context::<WorkspaceAccent>()`.
#[derive(Clone, PartialEq)]
pub struct WorkspaceAccent(pub String);

impl WorkspaceAccent {
    pub fn as_css_var(&self) -> String {
        format!("--color-workspace-accent: {}", self.0)
    }
}

impl Default for WorkspaceAccent {
    fn default() -> Self {
        Self(String::from("#2e7d32"))
    }
}
