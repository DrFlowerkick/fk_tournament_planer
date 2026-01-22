//! Sport Config Module

mod edit;
mod search;
mod select_sport;

use app_utils::{
    components::{banner::GlobalErrorBanner, toast::ToastContainer},
    params::SportParams,
    state::{error_state::PageErrorContext, toast_state::ToastContext},
};
pub use edit::SportConfigForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use leptos_router::nested_router::Outlet;
pub use search::SearchSportConfig;
pub use select_sport::SelectSportPlugin;

#[component]
pub fn SportConfigPage() -> impl IntoView {
    // ToDo: when we will migrate sport config into Home, this will not be required
    // set context for error reporting
    let page_error_context = PageErrorContext::new();
    provide_context(page_error_context);
    let toast_context = ToastContext::new();
    provide_context(toast_context);

    let sport_id_query = use_query::<SportParams>();

    view! {
        <GlobalErrorBanner />
        <ToastContainer />
        <div>
            <SelectSportPlugin />
        </div>
        <div class="my-4"></div>
        <div>
            {move || {
                if let Ok(sport_params) = sport_id_query.get() && sport_params.sport_id.is_some() {
                    view! { <SearchSportConfig /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>

        <div class="mt-4">
            <Outlet />
        </div>
    }
}
