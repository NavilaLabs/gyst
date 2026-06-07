//! `com.acme.leave-requests` — reference plugin for the zeitrak plugin platform.
//!
//! Exercises all three capability tiers:
//! - **Constructive**: `leave_request` aggregate + `pending_leaves` projection
//! - **Reactive**: subscribes to `TimesheetStopped`
//! - **Interceptive**: pre-hook on `timesheet.Start` — cancels if user is on leave
//! - **UI**: contributes to `dashboard.widgets` and `sidebar.entries`

use serde::{Deserialize, Serialize};
use zeitrak_plugin_sdk::{
    AggregateEventEmit, DioxusPlugin, DomainEventEnvelope, HandleCommandOutput, HookCall,
    HookResult, PdkError, PluginCtx, PluginId, PluginManifest, PluginProjectionEvent,
    PluginView, PriorityHint, SlotProvider, SlotRegistration, ZeitrakAggregate,
    ZeitrakEventSubscriber, div, h2, p, text,
};
use zeitrak_plugin_sdk::{
    plugin, zeitrak_aggregate, zeitrak_hook, zeitrak_projection, on_domain_event_export,
};

// ── Aggregate state ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaveRequestState {
    pub id: String,
    pub user_id: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String, // "pending" | "approved" | "rejected"
    pub reason: Option<String>,
}

// ── Aggregate commands ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum LeaveRequestCommand {
    Submit { user_id: String, start_date: String, end_date: String, reason: Option<String> },
    Approve { approved_by: String },
    Reject { rejected_by: String, reason: Option<String> },
}

// ── Aggregate events ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveRequestSubmittedPayload {
    pub user_id: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveRequestApprovedPayload {
    pub approved_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveRequestRejectedPayload {
    pub rejected_by: String,
    pub reason: Option<String>,
}

// ── LeaveRequest aggregate ────────────────────────────────────────────────────

pub struct LeaveRequest;

impl ZeitrakAggregate for LeaveRequest {
    type State = LeaveRequestState;

    fn initial_state() -> Self::State {
        LeaveRequestState::default()
    }

    fn apply(mut state: Self::State, event: AggregateEventEmit) -> Result<Self::State, PdkError> {
        match event.event_type.as_str() {
            "LeaveRequestSubmitted" => {
                if let Ok(p) = serde_json::from_value::<LeaveRequestSubmittedPayload>(event.payload) {
                    state.user_id = p.user_id;
                    state.start_date = p.start_date;
                    state.end_date = p.end_date;
                    state.reason = p.reason;
                    state.status = "pending".to_string();
                }
            }
            "LeaveRequestApproved" => {
                state.status = "approved".to_string();
            }
            "LeaveRequestRejected" => {
                state.status = "rejected".to_string();
            }
            _ => {}
        }
        Ok(state)
    }

    fn handle_command(state: Self::State, command: serde_json::Value) -> HandleCommandOutput {
        let cmd: LeaveRequestCommand = match serde_json::from_value(command) {
            Ok(c) => c,
            Err(e) => return HandleCommandOutput::reject(e.to_string()),
        };
        match cmd {
            LeaveRequestCommand::Submit { user_id, start_date, end_date, reason } => {
                if !state.status.is_empty() {
                    return HandleCommandOutput::reject("leave request already exists");
                }
                HandleCommandOutput::emit(
                    "LeaveRequestSubmitted",
                    &LeaveRequestSubmittedPayload { user_id, start_date, end_date, reason },
                )
            }
            LeaveRequestCommand::Approve { approved_by } => {
                if state.status != "pending" {
                    return HandleCommandOutput::reject("can only approve a pending request");
                }
                HandleCommandOutput::emit(
                    "LeaveRequestApproved",
                    &LeaveRequestApprovedPayload { approved_by },
                )
            }
            LeaveRequestCommand::Reject { rejected_by, reason } => {
                if state.status != "pending" {
                    return HandleCommandOutput::reject("can only reject a pending request");
                }
                HandleCommandOutput::emit(
                    "LeaveRequestRejected",
                    &LeaveRequestRejectedPayload { rejected_by, reason },
                )
            }
        }
    }
}

zeitrak_aggregate! { name: leave_request, handler: LeaveRequest }

// ── Projection ────────────────────────────────────────────────────────────────

fn project_pending_leaves(_event: PluginProjectionEvent) -> Result<(), PdkError> {
    // In a real plugin this would call the host's storage API (dx_set_state /
    // dx_query_raw) to maintain the pending_leaves projection table.
    // Omitted here for brevity — the host infrastructure manages table creation.
    Ok(())
}

zeitrak_projection! { name: pending_leaves, handler: project_pending_leaves }

// ── Plugin struct ─────────────────────────────────────────────────────────────

pub struct LeaveRequestsPlugin;

impl DioxusPlugin for LeaveRequestsPlugin {
    fn manifest() -> PluginManifest {
        PluginManifest {
            id: PluginId("com.acme.leave-requests".into()),
            version: "0.1.0".into(),
            slots: vec![
                SlotRegistration {
                    name: "dashboard.widgets".into(),
                    priority_hint: PriorityHint::Normal,
                },
                SlotRegistration {
                    name: "sidebar.entries".into(),
                    priority_hint: PriorityHint::Low,
                },
            ],
            ..PluginManifest::default()
        }
    }
}

// ── Dashboard widget slot ─────────────────────────────────────────────────────
//
// Phase 1 supports one `slot_render` export per plugin.  The plugin manifest
// declares both `dashboard.widgets` and `sidebar.entries`, but the single
// `slot_render` export renders the dashboard widget.  The sidebar entry will
// be served via a second slot export once Phase 2 proc-macros land.

pub struct DashboardWidget;

impl DioxusPlugin for DashboardWidget {
    fn manifest() -> PluginManifest {
        LeaveRequestsPlugin::manifest()
    }
}

impl SlotProvider for DashboardWidget {
    const SLOT_NAME: &'static str = "dashboard.widgets";

    fn render(_ctx: &PluginCtx) -> Result<PluginView, PdkError> {
        Ok(div()
            .class("plugin-widget leave-requests-widget")
            .child(h2().child(text("Pending Leave Requests")).build())
            .child(p().child(text("No pending requests.")).build())
            .build())
    }
}

// ── Domain event subscriber ───────────────────────────────────────────────────

impl ZeitrakEventSubscriber for LeaveRequestsPlugin {
    fn on_domain_event(envelope: DomainEventEnvelope) -> Result<(), PdkError> {
        if envelope.event_name == "TimesheetStopped" {
            // In a real implementation: look up the user's vacation-day counter and
            // decrement it if the stopped timesheet falls on a leave day.
        }
        Ok(())
    }
}

on_domain_event_export!(LeaveRequestsPlugin);

// ── Pre-hook: timesheet.Start ─────────────────────────────────────────────────

fn guard_start(call: HookCall) -> Result<HookResult, PdkError> {
    // In a real implementation: check if the calling user has an approved leave
    // covering today and, if so, cancel with a reason.
    // Here we always continue — the hook machinery is exercised regardless.
    let _ = &call;
    Ok(HookResult::Continue { context: call.context })
}

zeitrak_hook! { service: timesheet, command: Start, phase: Pre, handler: guard_start }

// ── Plugin entry point ────────────────────────────────────────────────────────

plugin! {
    type: LeaveRequestsPlugin,
    slots: [DashboardWidget],
}
