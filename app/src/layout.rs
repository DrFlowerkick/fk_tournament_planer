//! layout of the app

use crate::header::Header;
use app_utils::{
    components::{global_error_banner::GlobalErrorBanner, toast::ToastContainer},
    state::error_state::PageErrorContext,
};
use leptos::prelude::*;
use leptos_router::nested_router::Outlet;

#[component]
pub fn Layout() -> impl IntoView {
    // Get context needed for UI logic
    let page_err_ctx = expect_context::<PageErrorContext>();

    view! {
        <div class="flex flex-col min-h-screen">
            // navigation header is now part of the route tree
            <Header />

            <ToastContainer />

            <div class="sticky z-40 top-16 bg-base-200">
                <GlobalErrorBanner />
            </div>

            <main
                class="flex-grow p-4 bg-base-200 transition-all duration-200"
                class:opacity-50=move || page_err_ctx.has_errors()
                inert=move || page_err_ctx.has_errors()
            >
                // Hier werden die Child-Routes gerendert via Outlet
                <Outlet />
            </main>

            <footer class="footer footer-center p-4 bg-base-300 text-base-content">
                <div>
                    <p>"© 2025 FK-Tournament-Planer - Alle Rechte vorbehalten"</p>
                </div>
            </footer>
        </div>
    }
}
