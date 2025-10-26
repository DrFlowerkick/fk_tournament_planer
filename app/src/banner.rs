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
        <div
            data-testid="acknowledgment-banner"
            class="p-3 my-2 text-sm text-error-content bg-error rounded-lg"
            role="alert"
        >
            <p>{format!("{msg}")}</p>
            <button
                type="button"
                data-testid="btn-acknowledgment-action"
                class="btn btn-sm btn-outline mt-2"
                on:click=move |_| ack_action()
            >
                {format!("{ack_btn_text}")}
            </button>
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
        <div
            class="p-3 my-2 text-sm text-error-content bg-error rounded-lg"
            role="alert"
            data-testid="acknowledgment-navigate-banner"
        >
            <p>{format!("{msg}")}</p>
            <button
                class="btn btn-sm btn-outline mt-2"
                data-testid="btn-acknowledgment-navigate-action"
                on:click=move |_| ack_action()
            >
                {format!("{ack_btn_text}")}
            </button>
            <button
                class="btn btn-sm btn-outline mt-2 ml-2"
                data-testid="btn-acknowledgment-navigate"
                on:click=move |_| {
                    navigate(&navigate_url, NavigateOptions::default());
                }
            >
                {format!("{nav_btn_text}")}
            </button>
        </div>
    }
}
