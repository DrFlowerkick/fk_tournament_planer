//! handling persistent errors of application

use leptos::prelude::*;
use uuid::Uuid;

// --- Data Structures ---

#[derive(Clone)]
pub struct ErrorAction {
    pub label: String,
    pub on_click: Callback<()>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKey {
    /// Errors occurring during data reading (Resources: Load single, List many)
    Read,
    /// Errors occurring during data writing (Actions: Save, Update, Delete)
    Write,
    /// Catch-all for other errors
    General,
    /// Specific custom error keys
    Custom(String),
}

#[derive(Clone)]
pub struct ActiveError {
    pub component_id: Uuid,
    pub key: ErrorKey,
    pub message: String,
    /// If present, shows a primary action button (e.g. "Retry", "Reload")
    pub retry_action: Option<ErrorAction>,
    /// Mandatory secondary/cancel action (e.g. "Dismiss", "Back")
    pub cancel_action: ErrorAction,
}

impl ActiveError {
    /// Starts the construction of a new ActiveError using the Builder pattern.
    /// Returns a builder in 'NoCancelAction' state.
    pub fn builder(
        component_id: Uuid,
        message: impl Into<String>,
    ) -> ActiveErrorBuilder<NoCancelAction> {
        ActiveErrorBuilder::new(component_id, message)
    }
}

// --- Builder Implementation ---

// 1. State Markers (Zero-Sized Types)
pub struct NoCancelAction;
pub struct HasCancelAction;

// 2. The Builder Struct with State Parameter
pub struct ActiveErrorBuilder<State> {
    component_id: Uuid,
    key: ErrorKey,
    message: String,
    retry_action: Option<ErrorAction>,
    cancel_action: Option<ErrorAction>,
    _marker: std::marker::PhantomData<State>,
}

// 3. Implementation for Initial State
impl ActiveErrorBuilder<NoCancelAction> {
    fn new(component_id: Uuid, message: impl Into<String>) -> Self {
        Self {
            component_id,
            key: ErrorKey::General,
            message: message.into(),
            retry_action: None,
            cancel_action: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the mandatory cancel/dismiss action.
    /// Transitions the builder state from NoCancelAction to HasCancelAction.
    pub fn with_cancel(
        self,
        label: impl Into<String>,
        on_click: Callback<()>,
    ) -> ActiveErrorBuilder<HasCancelAction> {
        ActiveErrorBuilder {
            component_id: self.component_id,
            key: self.key,
            message: self.message,
            retry_action: self.retry_action,
            cancel_action: Some(ErrorAction {
                label: label.into(),
                on_click,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the mandatory cancel/dismiss action.
    /// Transitions the builder state from NoCancelAction to HasCancelAction.
    pub fn with_clear_error_on_cancel(
        self,
        label: impl Into<String>,
    ) -> ActiveErrorBuilder<HasCancelAction> {
        let error_ctx = expect_context::<PageErrorContext>();
        let cancel_key = self.key.clone();
        ActiveErrorBuilder {
            component_id: self.component_id,
            key: self.key,
            message: self.message,
            retry_action: self.retry_action,
            cancel_action: Some(ErrorAction {
                label: label.into(),
                on_click: Callback::new(move |()| {
                    error_ctx.clear_error(self.component_id, cancel_key.clone());
                }),
            }),
            _marker: std::marker::PhantomData,
        }
    }
}

// 4. Methods available in ANY state (both NoCancel and HasCancel)
impl<State> ActiveErrorBuilder<State> {
    /// Overrides the default ErrorKey::General.
    pub fn with_key(mut self, key: ErrorKey) -> Self {
        self.key = key;
        self
    }

    /// Adds or replaces a primary retry action.
    pub fn with_retry(mut self, label: impl Into<String>, on_click: Callback<()>) -> Self {
        self.retry_action = Some(ErrorAction {
            label: label.into(),
            on_click,
        });
        self
    }
}

// 5. Finalization - ONLY available if cancel_action was set
impl ActiveErrorBuilder<HasCancelAction> {
    // Requirement 2: Compile-time safety instead of runtime expect
    pub fn build(self) -> ActiveError {
        ActiveError {
            component_id: self.component_id,
            key: self.key,
            message: self.message,
            retry_action: self.retry_action,
            // Safe unwrap because HasCancelAction state guarantees it is Some
            cancel_action: self.cancel_action.unwrap(),
        }
    }
}

// --- Context ---

#[derive(Clone, Copy)]
pub struct PageErrorContext(RwSignal<Vec<ActiveError>>);

impl PageErrorContext {
    pub fn new() -> Self {
        Self(RwSignal::new(Vec::new()))
    }

    /// Report an error. Updates existing error if (component_id, key) matches.
    pub fn report_error(&self, new_error: ActiveError) {
        self.0.update(|list| {
            if let Some(existing) = list
                .iter_mut()
                .find(|e| e.component_id == new_error.component_id && e.key == new_error.key)
            {
                *existing = new_error;
            } else {
                list.push(new_error);
            }
        });
    }

    /// Removes a specific error.
    pub fn clear_error(&self, component_id: Uuid, key: ErrorKey) {
        self.0.update(|list| {
            list.retain(|e| !(e.component_id == component_id && e.key == key));
        });
    }

    /// Removes all errors for a specific component (e.g. on cleanup).
    pub fn clear_all_for_component(&self, component_id: Uuid) {
        self.0.update(|list| {
            list.retain(|e| e.component_id != component_id);
        });
    }

    /// Executes all retry actions present in the current error list.
    /// Used for the "Global Retry" button.
    pub fn retry_all(&self) {
        // We clone actions to avoid holding the lock during execution
        let actions: Vec<_> = self.0.with(|list| {
            list.iter()
                .filter_map(|e| e.retry_action.as_ref().map(|a| a.on_click.clone()))
                .collect()
        });

        for callback in actions {
            // We assume actions are safe and don't panic.
            // If they modify the error list (e.g. clear error on start), that's fine.
            callback.run(());
        }
    }

    /// Returns a read-only reactive signal of the errors.
    pub fn errors(&self) -> Signal<Vec<ActiveError>> {
        self.0.into()
    }

    /// Helper that efficiently retrieves the first active error (cloned)
    pub fn get_first_error(&self) -> Option<ActiveError> {
        self.0.with(|list| list.first().cloned())
    }

    /// Checks if there are any errors active.
    pub fn has_errors(&self) -> bool {
        !self.0.with(Vec::is_empty)
    }
}
