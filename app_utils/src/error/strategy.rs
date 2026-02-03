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
    toast_ctx: &ToastContext, // NEU: Parameter
    component_id: Uuid,
    error: &AppError,
    retry_fn: Callback<()>,
) {
    let key = ErrorKey::Write;

    match error {
        // 1. Optimistic Lock Conflict -> Banner
        AppError::Core(CoreError::Db(DbError::OptimisticLockConflict)) => {
            let builder = ActiveError::builder(
                component_id,
                "The record has been modified in the meantime. Please reload.",
            )
            .with_key(key.clone())
            .with_retry("Reload & Overwrite", retry_fn)
            .with_clear_error_on_cancel("Cancel");

            page_ctx.report_error(builder.build());
        }

        // 2a. Resource Not Found -> Banner
        AppError::ResourceNotFound(entity, _) => {
            let msg = format!("'{entity}' does not exist anymore.");

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Reload to fix", retry_fn)
                .with_clear_error_on_cancel("Cancel");

            page_ctx.report_error(builder.build());
        }

        // 2b. Db Generic Not Found -> Banner
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let msg = "The requested record was not found.";

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Reload to fix", retry_fn)
                .with_clear_error_on_cancel("Cancel");

            page_ctx.report_error(builder.build());
        }

        // 3. Unique Violation -> Banner (Dismissible)
        AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) => {
            let msg = field_opt
                .as_ref()
                .map(|f| format!("The value for '{f}' is already in use."))
                .unwrap_or_else(|| "A unique value is already in use.".to_string());

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_clear_error_on_cancel("OK");

            page_ctx.report_error(builder.build());
        }

        // 4. FK / Check Violation -> Banner
        AppError::Core(CoreError::Db(DbError::ForeignKeyViolation(_)))
        | AppError::Core(CoreError::Db(DbError::CheckViolation(_))) => {
            let builder = ActiveError::builder(component_id, "Inconsistent data operation.")
                .with_key(key.clone())
                .with_retry("Refresh Data", retry_fn)
                .with_clear_error_on_cancel("Close");

            page_ctx.report_error(builder.build());
        }

        // 5. Config Errors -> Banner
        AppError::Core(CoreError::Sport(app_core::SportError::InvalidJsonConfig(_))) => {
            let builder = ActiveError::builder(
                component_id,
                "System Data Error: Invalid Configuration. Please contact support.",
            )
            .with_key(key.clone())
            .with_clear_error_on_cancel("Close");

            page_ctx.report_error(builder.build());
        }

        // 6. Everything else -> TOAST
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
    // Renamed to clarify intent: This must navigate to a safe place (Dashboard)
    go_home_fn: Callback<()>,
) {
    let key = ErrorKey::Read;

    match error {
        // Case 1: Specific Entity not found
        AppError::ResourceNotFound(entity, _) => {
            let msg = format!("'{entity}' could not be found.");

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Retry", retry_fn)
                .with_cancel("To Dashboard", go_home_fn);

            ctx.report_error(builder.build());
        }

        // Case 2: Generic Database Not Found
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let msg = "The requested data could not be found.".to_string();

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Retry", retry_fn)
                .with_cancel("To Dashboard", go_home_fn);

            ctx.report_error(builder.build());
        }

        // Case 3: All other errors (treat as fatal/blocker for loading)
        _ => {
            let builder = ActiveError::builder(component_id, "Data could not be loaded.")
                .with_key(key.clone())
                .with_retry("Try again", retry_fn)
                .with_cancel("To Dashboard", go_home_fn);
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
    go_home_fn: Callback<()>,
) {
    let key = ErrorKey::General;

    let error_msg = error_msg.into();

    let mut builder = ActiveError::builder(component_id, error_msg).with_key(key);

    if let Some(retry) = retry_fn {
        builder = builder.with_retry("Retry", retry);
    }

    let builder = builder.with_cancel("To Dashboard", go_home_fn);

    ctx.report_error(builder.build());
}
