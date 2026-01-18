use crate::state::toast_state::{ToastContext, ToastVariant};
use leptos::prelude::*;

#[component]
pub fn ToastContainer() -> impl IntoView {
    // Try to get context, panic if missing (safe approach)
    let ctx = expect_context::<ToastContext>();
    let toasts = ctx.list();

    view! {
        // DaisyUI 'toast' class positions the container fixed
        // 'toast-end' = right, 'toast-bottom' = bottom (or toast-top)
        <div class="toast toast-end toast-bottom z-50 flex flex-col gap-2">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    let alert_class = match toast.variant {
                        ToastVariant::Info => "alert-info",
                        ToastVariant::Success => "alert-success",
                        ToastVariant::Warning => "alert-warning",
                        ToastVariant::Error => "alert-error",
                    };

                    view! {
                        // Animations (fade-in) could be done via CSS
                        <div class=format!("alert {} shadow-lg min-w-[300px]", alert_class)>
                            // Icon based on type (optional)
                            <span>{toast.message}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}
