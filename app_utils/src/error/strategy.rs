use crate::{
    error::AppError,
    state::{
        error_state::{ActiveError, ErrorKey, PageErrorContext},
        toast_state::ToastContext,
    },
};
use app_core::{CoreError, DbError};
use uuid::Uuid;

/// Evaluates a save/action error (Write).
/// - Known critical errors -> PageErrorContext (Banner)
/// - Transient/Technical errors -> ToastContext (Popup)
pub fn handle_write_error(
    page_ctx: &PageErrorContext,
    toast_ctx: &ToastContext, // NEU: Parameter
    component_id: Uuid,
    error: &AppError,
    retry_fn: impl Fn() + 'static + Send + Sync + Clone,
) {
    let key = ErrorKey::Write;

    match error {
        // 1. Optimistic Lock Conflict -> Banner
        AppError::Core(CoreError::Db(DbError::OptimisticLockConflict)) => {
            let retry_clone = retry_fn.clone();
            let builder = ActiveError::builder(
                component_id,
                "The record has been modified in the meantime. Please reload.",
            )
            .with_key(key.clone())
            .with_retry("Reload & Overwrite", move || retry_clone());

            let ctx_cancel = *page_ctx;
            let builder = builder.with_cancel("Cancel", move || {
                ctx_cancel.clear_error(component_id, key.clone());
            });

            page_ctx.report_error(builder.build());
        }

        // 2a. Resource Not Found -> Banner
        AppError::ResourceNotFound(entity, _) => {
            let retry_clone = retry_fn.clone();
            let msg = format!("'{entity}' does not exist anymore.");

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Reload to fix", move || retry_clone());

            let ctx_cancel = *page_ctx;
            let builder = builder.with_cancel("Close", move || {
                ctx_cancel.clear_error(component_id, key.clone());
            });

            page_ctx.report_error(builder.build());
        }

        // 2b. Db Generic Not Found -> Banner
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let retry_clone = retry_fn.clone();
            let msg = "The requested record was not found.";

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Reload to fix", move || retry_clone());

            let ctx_cancel = *page_ctx;
            let builder = builder.with_cancel("Close", move || {
                ctx_cancel.clear_error(component_id, key.clone());
            });

            page_ctx.report_error(builder.build());
        }

        // 3. Unique Violation -> Banner (Dismissible)
        AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) => {
            let msg = field_opt
                .as_ref()
                .map(|f| format!("The value for '{f}' is already in use."))
                .unwrap_or_else(|| "A unique value is already in use.".to_string());

            let ctx_cancel = *page_ctx;
            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_cancel("OK", move || {
                    ctx_cancel.clear_error(component_id, key.clone());
                });

            page_ctx.report_error(builder.build());
        }

        // 4. FK / Check Violation -> Banner
        AppError::Core(CoreError::Db(DbError::ForeignKeyViolation(_)))
        | AppError::Core(CoreError::Db(DbError::CheckViolation(_))) => {
            let retry_clone = retry_fn.clone();
            let builder = ActiveError::builder(component_id, "Inconsistent data operation.")
                .with_key(key.clone())
                .with_retry("Refresh Data", move || retry_clone());

            let ctx_cancel = *page_ctx;
            let builder = builder.with_cancel("Close", move || {
                ctx_cancel.clear_error(component_id, key.clone());
            });

            page_ctx.report_error(builder.build());
        }

        // 5. Config Errors -> Banner
        AppError::Core(CoreError::Sport(app_core::SportError::InvalidJsonConfig(_))) => {
            let ctx_cancel = *page_ctx;
            let builder = ActiveError::builder(
                component_id,
                "System Data Error: Invalid Configuration. Please contact support.",
            )
            .with_key(key.clone())
            .with_cancel("Close", move || {
                ctx_cancel.clear_error(component_id, key.clone());
            });

            page_ctx.report_error(builder.build());
        }

        // 6. Everything else -> TOAST (NEU!)
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
    retry_fn: impl Fn() + 'static + Send + Sync + Clone,
    // Renamed to clarify intent: This must navigate to a safe place (Dashboard)
    go_home_fn: impl Fn() + 'static + Send + Sync + Clone,
) {
    let key = ErrorKey::Read;

    match error {
        // Case 1: Specific Entity not found
        AppError::ResourceNotFound(entity, _) => {
            let retry_clone = retry_fn.clone();
            let msg = format!("'{entity}' could not be found.");

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Retry", move || retry_clone())
                .with_cancel("To Dashboard", move || go_home_fn());

            ctx.report_error(builder.build());
        }

        // Case 2: Generic Database Not Found
        AppError::Core(CoreError::Db(DbError::NotFound)) => {
            let retry_clone = retry_fn.clone();
            let msg = "The requested data could not be found.".to_string();

            let builder = ActiveError::builder(component_id, msg)
                .with_key(key.clone())
                .with_retry("Retry", move || retry_clone())
                .with_cancel("To Dashboard", move || go_home_fn());

            ctx.report_error(builder.build());
        }

        // Case 3: All other errors (treat as fatal/blocker for loading)
        _ => {
            let retry_clone = retry_fn.clone();
            let builder = ActiveError::builder(component_id, "Data could not be loaded.")
                .with_key(key.clone())
                .with_retry("Try again", move || retry_clone())
                .with_cancel("To Dashboard", move || go_home_fn());

            ctx.report_error(builder.build());
        }
    }
}
