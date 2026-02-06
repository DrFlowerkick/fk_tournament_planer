use leptos::prelude::*;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub id: Uuid,
    pub message: String,
    pub variant: ToastVariant,
}

#[derive(Clone, Copy)]
pub struct ToastContext(RwSignal<Vec<Toast>>);

impl ToastContext {
    pub fn new() -> Self {
        Self(RwSignal::new(Vec::new()))
    }

    /// Returns a read-only signal for the UI
    pub fn list(&self) -> Signal<Vec<Toast>> {
        self.0.into()
    }

    fn add(&self, message: impl Into<String>, variant: ToastVariant) {
        let msg_string = message.into();

        // 1. Deduplication: Check if exactly this message is already displayed.
        // If yes: Abort (or we could reset the timer,
        // but ignoring is effective enough for "spam prevention").
        let already_exists = self.0.with(|list| {
            list.iter()
                .any(|t| t.message == msg_string && t.variant == variant)
        });

        if already_exists {
            return;
        }

        let new_id = Uuid::new_v4();
        let toast = Toast {
            id: new_id,
            message: msg_string,
            variant,
        };

        // 2. Add
        self.0.update(|list| list.push(toast));

        // 3. Auto-Remove Timer (5 seconds)
        // We use set_timeout without handle as simple "fire & forget"
        // logic is sufficient here.
        let ctx = *self;
        set_timeout(
            move || {
                ctx.remove(new_id);
            },
            Duration::from_secs(5),
        );
    }

    pub fn remove(&self, id: Uuid) {
        self.0.update(|list| {
            list.retain(|t| t.id != id);
        });
    }

    // Helper Methods for convenience
    pub fn info(&self, msg: impl Into<String>) {
        self.add(msg, ToastVariant::Info);
    }

    pub fn success(&self, msg: impl Into<String>) {
        self.add(msg, ToastVariant::Success);
    }

    pub fn warning(&self, msg: impl Into<String>) {
        self.add(msg, ToastVariant::Warning);
    }

    pub fn error(&self, msg: impl Into<String>) {
        self.add(msg, ToastVariant::Error);
    }
}
