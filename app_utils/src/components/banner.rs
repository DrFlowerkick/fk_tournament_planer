use crate::state::error_state::PageErrorContext;
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use std::fmt::Display;

#[component]
pub fn AcknowledgmentBanner(
    msg: impl Display,
    ack_btn_text: impl Display,
    ack_action: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div data-testid="acknowledgment-banner" class="alert alert-warning" role="alert">
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
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
            </svg>
            <span>{format!("{msg}")}</span>
            <div>
                <button
                    type="button"
                    data-testid="btn-acknowledgment-action"
                    class="btn btn-sm btn-outline"
                    on:click=move |_| ack_action()
                >
                    {format!("{ack_btn_text}")}
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn AcknowledgmentAndNavigateBanner(
    msg: impl Display,
    ack_btn_text: impl Display,
    ack_action: impl Fn() + 'static,
    nav_btn_text: impl Display,
    navigate_url: String,
) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div class="alert alert-error" role="alert" data-testid="acknowledgment-navigate-banner">
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
            <span>{format!("{msg}")}</span>
            <div>
                <button
                    class="btn btn-sm btn-outline"
                    data-testid="btn-acknowledgment-navigate-action"
                    on:click=move |_| ack_action()
                >
                    {format!("{ack_btn_text}")}
                </button>
                <button
                    class="btn btn-sm btn-outline ml-2"
                    data-testid="btn-acknowledgment-navigate"
                    on:click=move |_| {
                        navigate(&navigate_url, NavigateOptions::default());
                    }
                >
                    {format!("{nav_btn_text}")}
                </button>
            </div>
        </div>
    }
}

/// A global error banner that displays the first active error from the PageErrorContext.
#[component]
pub fn GlobalErrorBanner() -> impl IntoView {
    let ctx = use_context::<PageErrorContext>().expect("No Error Context found");

    view! {
        <Show when=move || ctx.has_errors() fallback=|| ()>
            {move || {
                if let Some(first_error) = ctx.get_first_error() {
                    view! {
                        <div class="alert alert-error">
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
                                {match &first_error.retry_action {
                                    Some(action) => {
                                        let label = action.label.clone();
                                        view! {
                                            <button
                                                class="btn btn-sm btn-ghost"
                                                on:click=move |_| ctx.retry_all()
                                            >
                                                {label}
                                            </button>
                                        }
                                            .into_any()
                                    }
                                    None => ().into_any(),
                                }}
                                <button
                                    class="btn btn-sm"
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
