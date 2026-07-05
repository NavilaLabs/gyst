//! A reusable `statig` state machine for async form submission.
//!
//! Usage in a Dioxus component:
//!
//! ```rust,ignore
//! use crate::form_machine::{FormAction, FormMachine, State, new_form};
//!
//! let mut form = use_signal(new_form);
//!
//! let on_submit = move |_| async move {
//!     form.write().handle(&FormAction::Submit);
//!     match api::customer::create_customer(...).await {
//!         Ok(_)  => form.write().handle(&FormAction::Succeed("Created!".into())),
//!         Err(e) => form.write().handle(&FormAction::Fail(e.to_string())),
//!     }
//! };
//!
//! // In RSX — check state with matches!, access message via Deref to FormMachine:
//! //   matches!(form.read().state(), State::Submitting {})
//! //   form.read().message  — populated on Succeed / Fail
//! ```

pub use statig::blocking::StateMachine;
use statig::prelude::*;

/// Actions that drive form state transitions.
///
/// Named `FormAction` (not `FormEvent`) to avoid shadowing Dioxus's
/// `FormEvent` type alias in components that import both.
#[derive(Clone, PartialEq, Debug)]
pub enum FormAction {
    /// The user triggered a submission.
    Submit,
    /// The async operation completed successfully; carries a display message.
    Succeed(String),
    /// The async operation failed; carries the error message.
    Fail(String),
    /// Reset the form back to the idle state.
    Reset,
}

/// Context held by the machine between transitions.
///
/// `message` is populated by `Succeed` and `Fail` events so that templates
/// can render success/error feedback without inspecting the event payload.
#[derive(Default)]
pub struct FormMachine {
    pub message: String,
}

#[statig::state_machine(initial = "State::idle()", state(derive(Clone, PartialEq, Debug)))]
impl FormMachine {
    /// Waiting for user interaction.
    #[state]
    fn idle(&mut self, event: &FormAction) -> Response<State> {
        match event {
            FormAction::Submit => Transition(State::submitting()),
            _ => Super,
        }
    }

    /// An async operation is in flight.
    #[state]
    fn submitting(&mut self, event: &FormAction) -> Response<State> {
        match event {
            FormAction::Succeed(msg) => {
                self.message = msg.clone();
                Transition(State::success())
            }
            FormAction::Fail(msg) => {
                self.message = msg.clone();
                Transition(State::error())
            }
            _ => Super,
        }
    }

    /// The last submission succeeded. `self.message` holds the success text.
    #[state]
    fn success(&mut self, event: &FormAction) -> Response<State> {
        match event {
            FormAction::Submit => {
                self.message.clear();
                Transition(State::submitting())
            }
            FormAction::Reset => {
                self.message.clear();
                Transition(State::idle())
            }
            _ => Super,
        }
    }

    /// The last submission failed. `self.message` holds the error text.
    #[state]
    fn error(&mut self, event: &FormAction) -> Response<State> {
        match event {
            FormAction::Submit => Transition(State::submitting()),
            FormAction::Reset => {
                self.message.clear();
                Transition(State::idle())
            }
            _ => Super,
        }
    }
}

/// Create a new, lazily-initialized form state machine.
///
/// Pass this directly to `use_signal`:
/// ```rust,ignore
/// let mut form = use_signal(new_form);
/// ```
///
/// The machine initializes itself automatically on the first `handle()` call.
pub fn new_form() -> StateMachine<FormMachine> {
    FormMachine::default().state_machine()
}
