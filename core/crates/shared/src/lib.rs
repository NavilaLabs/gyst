use std::fmt::Display;

use url::Url;

pub mod test_lifecycle {
    pub fn before() {
        dotenvy::from_filename_override(".env.test").expect("Failed to load .env.test.");
    }

    pub fn after() {
        dotenvy::from_filename_override(".env.dev").ok();
    }
}

pub fn extract_tenant_token_from_url(url: &str) -> Option<String> {
    let url = url.parse::<Url>().ok()?;
    url.path_segments()
        .and_then(|segments| segments.last())
        .map(|segment| segment.to_string())
        .and_then(|segment| segment.split("_").last().map(|token| token.to_string()))
}

#[test]
fn test_extract_tenant_token_from_url() {
    let url = "postgres://localhost:5432/loom_tenant_token";
    let token = extract_tenant_token_from_url(url);
    assert_eq!(token, Some("token".to_string()));

    let url = "sqlite:///workspaces/loom/loom_tenant_token";
    let token = extract_tenant_token_from_url(url);
    assert_eq!(token, Some("token".to_string()));
}
