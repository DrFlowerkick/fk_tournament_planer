use crate::{
    error::{AppError, ComponentError},
    state::{
        LabeledAction,
        error_state::{ActiveError, ErrorKey, PageErrorContext},
        toast_state::ToastContext,
    },
};
use leptos::prelude::*;
use uuid::Uuid;

/// handles app errors with a toast message
pub fn handle_with_toast(
    toast_ctx: &ToastContext,
    error: &AppError,
    interactive: Option<LabeledAction>,
) {
    let msg = error.to_web_ui_msg();

    toast_ctx.error(msg, interactive);
}

/// handle app errors with error banner and navigation back to a safe place (e.g. dashboard)
/// Components may register a retry handler for the error, which will be used in the error banner if available.
pub fn handle_with_error_banner(
    ctx: &PageErrorContext,
    error: &ComponentError,
    // This must navigate "back" to a safe place (e.g. Dashboard)
    back_fn: Callback<()>,
) {
    let key = ErrorKey::Read;
    let retry_fn = ctx.get_retry_handler(error.component_id);

    let msg = error.app_error.to_web_ui_msg();

    let mut builder = ActiveError::builder(error.component_id, key.clone(), msg);
    if let Some(retry_fn) = retry_fn {
        builder = builder.with_retry("Retry", retry_fn);
    }
    let builder = builder.with_cancel("Back", back_fn);

    ctx.report_error(builder.build());
}

/// Handles general errors for other operations (not specifically read or write).
pub fn handle_unexpected_ui_error(
    ctx: &PageErrorContext,
    component_id: Uuid,
    error_msg: impl Into<String>,
    back_fn: Callback<()>,
) {
    let key = ErrorKey::General;

    let error_msg = error_msg.into();
    let retry_fn = ctx.get_retry_handler(component_id);

    let mut builder =
        ActiveError::builder(component_id, key.clone(), error_msg).with_cancel("Back", back_fn);
    if let Some(retry_fn) = retry_fn {
        builder = builder.with_retry("Retry", retry_fn);
    }

    ctx.report_error(builder.build());
}
