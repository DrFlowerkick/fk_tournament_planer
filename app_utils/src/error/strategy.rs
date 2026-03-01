use crate::{
    error::{AppError, ComponentError},
    state::{
        error_state::{ActiveError, ErrorKey, PageErrorContext},
        toast_state::ToastContext,
    },
};
use app_core::{CoreError, DbError};
use leptos::prelude::*;
use uuid::Uuid;

/// Evaluates a save/action error (Write).
/// Since we use autosave and auto update, all save errors are reported via ToastContext (Popup)
pub fn handle_write_error(toast_ctx: &ToastContext, error: &AppError) {
    match error {
        // 1. Optimistic Lock Conflict -> Toast
        // The client registry and auto saving ensures, that always the latest version is loaded. If a version mismatch
        // occurs during saving, it means that parallel editing is happening. In this case, we still reload "automatically"
        // the current version. Therefore a manual reload by the user is not necessary
        // We inform the user about the parallel editing via a toast.
        // This should not happen often.
        AppError::Core(CoreError::Db(DbError::OptimisticLockConflict)) => {
            let msg = format!("{error}");
            toast_ctx.error(msg);
        }

        // 2. Unique Violation -> Toast
        // Validation error: Input needs correction (e.g. "Name already taken").
        AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) => {
            let msg = field_opt
                .as_ref()
                .map(|f| format!("A unique value is already in use: '{f}'."))
                .unwrap_or_else(|| "A unique value is already in use.".to_string());

            toast_ctx.error(msg);
        }

        // 3. Check Violation -> Toast
        // Validation error: Database constraint failed (e.g. "age >= 0").
        // Treated like UniqueViolation: The user must correct the input.
        AppError::Core(CoreError::Db(DbError::CheckViolation(constraint_opt))) => {
            let msg = constraint_opt
                .as_ref()
                .map(|c| format!("Data validation failed (Constraint: {}).", c))
                .unwrap_or_else(|| "Data validation failed.".to_string());

            toast_ctx.error(msg);
        }

        // 4. Everything else -> TOAST
        _ => {
            // "Fire & Forget" Toast
            // AppError implements Display via thiserror, so error.to_string() works fine.
            toast_ctx.error(error.to_string());
        }
    }
}

/// Evaluates a load and list error (Read) and updates the PageErrorContext.
/// Since read errors usually mean the page is broken, the cancel action implies
/// leaving the broken state (Navigation).
pub fn handle_read_error(
    ctx: &PageErrorContext,
    error: &ComponentError,
    // This must navigate "back" to a safe place (e.g. Dashboard)
    back_fn: Callback<()>,
) {
    let key = ErrorKey::Read;
    let retry_fn = ctx.get_retry_handler(error.component_id);

    match &error.app_error {
        // Case 1: Specific Entity not found
        AppError::ResourceNotFound(entity, _) => {
            let msg = format!("'{entity}' could not be found.");

            let mut builder = ActiveError::builder(error.component_id, key.clone(), msg);
            if let Some(retry_fn) = retry_fn {
                builder = builder.with_retry("Retry", retry_fn);
            }
            let builder = builder.with_cancel("Back", back_fn);

            ctx.report_error(builder.build());
        }

        // Case 2: Generic Database Not Found
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let msg = "The requested data could not be found in database.".to_string();

            let mut builder = ActiveError::builder(error.component_id, key.clone(), msg);
            if let Some(retry_fn) = retry_fn {
                builder = builder.with_retry("Retry", retry_fn);
            }
            let builder = builder.with_cancel("Back", back_fn);

            ctx.report_error(builder.build());
        }

        // Case 3: All other errors (treat as fatal/blocker for loading)
        err => {
            let mut builder =
                ActiveError::builder(error.component_id, key.clone(), err.to_string());
            if let Some(retry_fn) = retry_fn {
                builder = builder.with_retry("Retry", retry_fn);
            }
            let builder = builder.with_cancel("Back", back_fn);
            ctx.report_error(builder.build());
        }
    }
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
