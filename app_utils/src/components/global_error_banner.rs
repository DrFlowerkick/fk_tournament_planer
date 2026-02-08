use crate::state::error_state::PageErrorContext;
use leptos::prelude::*;

/// A global error banner that displays the first active error from the PageErrorContext.
#[component]
pub fn GlobalErrorBanner() -> impl IntoView {
    let ctx = expect_context::<PageErrorContext>();

    view! {
        <Show when=move || ctx.has_errors() fallback=|| ()>
            {move || {
                if let Some(first_error) = ctx.get_first_error() {
                    view! {
                        <div
                            class="alert alert-error"
                            data-testid="global-error-banner"
                            role="alert"
                        >
                            // The SVG is a "X" icon for visual error indication
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="stroke-current shrink-0 h-6 w-6"
                                fill="none"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                            </svg>

                            <span>{first_error.message.clone()}</span>

                            <div class="flex gap-2">
                                {first_error
                                    .retry_action
                                    .map(|action| {
                                        let label = action.label.clone();
                                        view! {
                                            <button
                                                class="btn btn-sm btn-ghost"
                                                data-testid="btn-retry-action"
                                                on:click=move |_| ctx.retry_all()
                                            >
                                                {label}
                                            </button>
                                        }
                                    })}
                                <button
                                    class="btn btn-sm"
                                    data-testid="btn-cancel-action"
                                    on:click=move |_| first_error.cancel_action.on_click.run(())
                                >
                                    {first_error.cancel_action.label.clone()}
                                </button>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </Show>
    }
}
