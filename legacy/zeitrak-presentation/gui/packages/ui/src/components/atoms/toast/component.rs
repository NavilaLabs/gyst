use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, PartialEq, Debug)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastVariant {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToastMessage {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
}

impl ToastMessage {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            message: message.into(),
            variant: ToastVariant::Success,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            message: message.into(),
            variant: ToastVariant::Error,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            message: message.into(),
            variant: ToastVariant::Info,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            message: message.into(),
            variant: ToastVariant::Warning,
        }
    }
}

/// Shared toast signal. Inject with `use_context::<Toasts>()` and push messages to it.
pub type Toasts = Signal<Vec<ToastMessage>>;

/// Extension methods on [`Toasts`] for ergonomic message pushing.
///
/// Reduces `toasts.write().push(ToastMessage::error(msg))` to `toasts.push_error(msg)`.
/// This is the idiomatic Rust extension-trait pattern applied to a foreign type alias.
pub trait ToastExt {
    fn push_error(&mut self, msg: impl Into<String>);
    fn push_success(&mut self, msg: impl Into<String>);
    fn push_info(&mut self, msg: impl Into<String>);
    fn push_warning(&mut self, msg: impl Into<String>);
}

impl ToastExt for Toasts {
    fn push_error(&mut self, msg: impl Into<String>) {
        self.write().push(ToastMessage::error(msg));
    }
    fn push_success(&mut self, msg: impl Into<String>) {
        self.write().push(ToastMessage::success(msg));
    }
    fn push_info(&mut self, msg: impl Into<String>) {
        self.write().push(ToastMessage::info(msg));
    }
    fn push_warning(&mut self, msg: impl Into<String>) {
        self.write().push(ToastMessage::warning(msg));
    }
}

/// Renders the active toast stack fixed in the bottom-right corner.
/// Place this once in your top-level layout after providing the `Toasts` context.
#[component]
pub fn ToastStack() -> Element {
    let toasts: Toasts = use_context();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "toast-stack",
            for toast in toasts.read().iter() {
                {
                    let t = toast.clone();
                    rsx! {
                        ToastItem { key: "{t.id}", toast: t }
                    }
                }
            }
        }
    }
}

#[component]
fn ToastItem(toast: ToastMessage) -> Element {
    let mut toasts: Toasts = use_context();
    let id = toast.id;

    // Auto-dismiss after 4 s for success/info, 7 s for errors (WASM only).
    #[cfg(target_arch = "wasm32")]
    {
        let delay_ms: u32 = match toast.variant {
            ToastVariant::Error => 7000,
            _ => 4000,
        };
        use_future(move || async move {
            gloo_timers::future::TimeoutFuture::new(delay_ms).await;
            toasts.write().retain(|t| t.id != id);
        });
    }

    rsx! {
        div {
            class: "toast",
            "data-variant": toast.variant.as_str(),
            p { class: "toast-message", "{toast.message}" }
            button {
                class: "toast-dismiss",
                onclick: move |_| toasts.write().retain(|t| t.id != id),
                "×"
            }
        }
    }
}
