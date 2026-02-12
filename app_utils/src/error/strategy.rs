use crate::{
    error::AppError,
    state::{
        error_state::{ActiveError, ErrorKey, PageErrorContext},
        toast_state::ToastContext,
    },
};
use app_core::{CoreError, DbError};
use leptos::prelude::*;
use uuid::Uuid;

/// Evaluates a save/action error (Write).
/// - Known critical errors -> PageErrorContext (Banner)
/// - Transient/Technical errors -> ToastContext (Popup)
pub fn handle_write_error(
    page_ctx: &PageErrorContext,
    toast_ctx: &ToastContext,
    component_id: Uuid,
    error: &AppError,
    retry_fn: Callback<()>,
) {
    let key = ErrorKey::Write;

    match error {
        // 1. Optimistic Lock Conflict -> Banner
        // The local data is stale compared to the server. A reload is mandatory to sync state.
        AppError::Core(CoreError::Db(DbError::OptimisticLockConflict)) => {
            let builder = ActiveError::builder(
                component_id,
                key.clone(),
                "The record has been modified in the meantime. Please reload. Any unsaved changes will be lost.",
            )
            .with_retry("Reload", retry_fn)
            .with_clear_error_on_cancel("Cancel");

            page_ctx.report_error(builder.build());
        }

        // 2. ForeignKey Violation -> Banner
        // Usually implies referencing data that was deleted/changed elsewhere (Stale State).
        // This is a system consistency issue requiring data refresh.
        AppError::Core(CoreError::Db(DbError::ForeignKeyViolation(_))) => {
            let builder =
                ActiveError::builder(component_id, key.clone(), "Inconsistent data operation.")
                    .with_retry("Refresh Data", retry_fn)
                    .with_clear_error_on_cancel("Close");

            page_ctx.report_error(builder.build());
        }

        // 3. Unique Violation -> Toast
        // Validation error: Input needs correction (e.g. "Name already taken").
        AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) => {
            let msg = field_opt
                .as_ref()
                .map(|f| format!("A unique value is already in use: '{f}'."))
                .unwrap_or_else(|| "A unique value is already in use.".to_string());

            toast_ctx.error(msg);
        }

        // 4. Check Violation -> Toast
        // Validation error: Database constraint failed (e.g. "age >= 0").
        // Treated like UniqueViolation: The user must correct the input.
        AppError::Core(CoreError::Db(DbError::CheckViolation(constraint_opt))) => {
            let msg = constraint_opt
                .as_ref()
                .map(|c| format!("Data validation failed (Constraint: {}).", c))
                .unwrap_or_else(|| "Data validation failed.".to_string());

            toast_ctx.error(msg);
        }

        // 5. Everything else -> TOAST
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
    component_id: Uuid,
    error: &AppError,
    retry_fn: Callback<()>,
    // This must navigate "back" to a safe place (e.g. Dashboard)
    back_fn: Callback<()>,
) {
    let key = ErrorKey::Read;

    match error {
        // Case 1: Specific Entity not found
        AppError::ResourceNotFound(entity, _) => {
            let msg = format!("'{entity}' could not be found.");

            let builder = ActiveError::builder(component_id, key.clone(), msg)
                .with_retry("Retry", retry_fn)
                .with_cancel("Back", back_fn);

            ctx.report_error(builder.build());
        }

        // Case 2: Generic Database Not Found
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let msg = "The requested data could not be found in database.".to_string();

            let builder = ActiveError::builder(component_id, key.clone(), msg)
                .with_retry("Retry", retry_fn)
                .with_cancel("Back", back_fn);

            ctx.report_error(builder.build());
        }

        // Case 3: All other errors (treat as fatal/blocker for loading)
        err => {
            let builder = ActiveError::builder(component_id, key.clone(), err.to_string())
                .with_retry("Retry", retry_fn)
                .with_cancel("Back", back_fn);
            ctx.report_error(builder.build());
        }
    }
}

/// Handles general errors for other operations (not specifically read or write).
pub fn handle_general_error(
    ctx: &PageErrorContext,
    component_id: Uuid,
    error_msg: impl Into<String>,
    retry_fn: Option<Callback<()>>,
    back_fn: Callback<()>,
) {
    let key = ErrorKey::General;

    let error_msg = error_msg.into();

    let mut builder = ActiveError::builder(component_id, key.clone(), error_msg);

    if let Some(retry) = retry_fn {
        builder = builder.with_retry("Retry", retry);
    }

    let builder = builder.with_cancel("Back", back_fn);

    ctx.report_error(builder.build());
}
