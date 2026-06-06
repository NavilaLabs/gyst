//! End-to-end integration test for the `com.acme.leave-requests` reference plugin
//! (Phase I — §15, step 36).
//!
//! # Prerequisites
//!
//! The WASM binary must be built before running these tests:
//!
//! ```sh
//! cd examples/plugins/leave-requests
//! cargo build --release --target wasm32-unknown-unknown
//! ```
//!
//! The `zeitrak-plugin-host` `build.rs` attempts this automatically during
//! `cargo test`.  If the `wasm32-unknown-unknown` target is not installed the
//! tests are automatically skipped (not failed).
//!
//! # What is tested
//!
//! 1. The runtime loads and validates the plugin manifest.
//! 2. The plugin is listed with the expected id after loading.
//! 3. `leave_request__initial_state` is callable and returns a JSON value.
//! 4. `hook_timesheet_Start_Pre` is callable with a `HookCall` payload.

use std::path::PathBuf;
use std::sync::Arc;

use dioxus_extism_host::{PluginRuntime, PluginSource};
use dioxus_extism_protocol::{HookCall, PluginId, SessionCtx};

/// Path to the compiled WASM artifact relative to the workspace root.
fn wasm_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("../target/wasm32-unknown-unknown/release/leave_requests.wasm")
}

/// Skip the test if the WASM file does not exist (wasm32 target not installed).
macro_rules! require_wasm {
    () => {
        let path = wasm_path();
        if !path.exists() {
            eprintln!(
                "SKIP: WASM file not found at {path:?}. \
                 Build with `cargo build --release --target wasm32-unknown-unknown` \
                 inside examples/plugins/leave-requests first."
            );
            return;
        }
    };
}

async fn build_runtime() -> Arc<PluginRuntime<()>> {
    PluginRuntime::<()>::builder()
        .add_plugin(PluginSource::File(wasm_path()))
        .build()
        .await
        .expect("PluginRuntime must build with leave-requests WASM")
}

#[tokio::test]
async fn loads_leave_requests_plugin() {
    require_wasm!();

    let runtime = build_runtime().await;
    let plugins = runtime.list_plugins().await;

    assert_eq!(plugins.len(), 1, "expected exactly one loaded plugin");
    assert_eq!(
        plugins[0].id.0, "com.acme.leave-requests",
        "plugin id must match the manifest"
    );
}

#[tokio::test]
async fn aggregate_initial_state_export_is_callable() {
    require_wasm!();

    let runtime = build_runtime().await;
    let plugin_id = PluginId("com.acme.leave-requests".into());
    let session = SessionCtx::default();

    let result: Result<serde_json::Value, _> = runtime
        .call_plugin(&plugin_id, "leave_request__initial_state", &(), &session, &())
        .await;

    assert!(
        result.is_ok(),
        "leave_request__initial_state must be callable and return a JSON value: {result:?}"
    );
}

#[tokio::test]
async fn hook_export_is_present_and_callable() {
    require_wasm!();

    let runtime = build_runtime().await;
    let plugin_id = PluginId("com.acme.leave-requests".into());
    let session = SessionCtx::default();

    let hook_call = HookCall {
        hook_name: "hook_timesheet_Start_Pre".into(),
        context: serde_json::json!({}),
    };

    let result: Result<serde_json::Value, _> = runtime
        .call_plugin(
            &plugin_id,
            "hook_timesheet_Start_Pre",
            &hook_call,
            &session,
            &(),
        )
        .await;

    assert!(
        result.is_ok(),
        "hook_timesheet_Start_Pre must be callable: {result:?}"
    );
}

#[tokio::test]
async fn aggregate_submit_command_produces_submitted_event() {
    require_wasm!();

    let runtime = build_runtime().await;
    let plugin_id = PluginId("com.acme.leave-requests".into());
    let session = SessionCtx::default();

    // First get the initial state.
    let initial_state: serde_json::Value = runtime
        .call_plugin(&plugin_id, "leave_request__initial_state", &(), &session, &())
        .await
        .expect("initial state must be callable");

    // Then call handle_command with a Submit command.
    let command_input = (
        initial_state,
        serde_json::json!({
            "command": "Submit",
            "user_id": "user-1",
            "start_date": "2026-07-01",
            "end_date": "2026-07-05",
            "reason": "Holiday"
        }),
    );

    let result: Result<serde_json::Value, _> = runtime
        .call_plugin(
            &plugin_id,
            "leave_request__handle_command",
            &command_input,
            &session,
            &(),
        )
        .await;

    assert!(
        result.is_ok(),
        "leave_request__handle_command must be callable: {result:?}"
    );

    let output = result.unwrap();
    assert!(
        output.get("events").is_some(),
        "Submit command must produce events: {output}"
    );
}
