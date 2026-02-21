//! header component

use app_utils::{
    hooks::blur_active_element::blur_active_element, state::activity_tracker::ActivityTracker,
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_url};

#[component]
pub fn Header() -> impl IntoView {
    // navigation hooks
    let url = use_url();
    let root_with_query = Signal::derive(move || {
        let query = url.get().search_params().to_query_string();
        if query.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", query)
        }
    });

    // Get the activity tracker context to reactively toggle the inert state
    let activity_tracker = expect_context::<ActivityTracker>();

    // Signal to manage the mobile menu state
    let (menu_open, set_menu_open) = signal(false);

    view! {
        <header class="navbar bg-base-300 sticky top-0 z-50">
            <div class="flex-1">
                <A
                    href=move || root_with_query.get()
                    attr:class="btn btn-ghost normal-case text-xl"
                >
                    "Tournament Planner"
                </A>
            </div>
            // Group loading indicator and menu button together on the right
            <div class="flex-none flex items-center gap-3 px-2">
                <Show when=move || activity_tracker.is_active.get()>
                    <span class="loading loading-bars loading-sm"></span>
                </Show>
                <div class="dropdown dropdown-end" class:dropdown-open=menu_open>
                    // Use a button instead of label/input to avoid event conflicts in Leptos.
                    // The 'swap-active' class controls which icon is visible based on the signal.
                    // Trigger blur if false or when clicking links to ensure closing the dropdown menu.
                    // This is required because daisyUI's dropdown relies on focus/blur of CSS selectors.
                    <button
                        type="button"
                        class="btn btn-ghost btn-circle swap swap-rotate"
                        class:swap-active=menu_open
                        on:click=move |_| {
                            set_menu_open.update(|v| *v = !*v);
                            if !menu_open.get() {
                                blur_active_element();
                            }
                        }
                        on:blur=move |_| set_menu_open.set(false)
                    >
                        // Hamburger menu icon (visible when menu_open is false)
                        <span class="swap-off icon-[heroicons--bars-3] w-6 h-6 inline-block"></span>

                        // Close icon (visible when menu_open is true)
                        <span class="swap-on icon-[heroicons--x-mark] w-6 h-6 inline-block"></span>
                    </button>

                    // Vertical dropdown menu
                    <ul
                        tabindex="0"
                        class="dropdown-content menu bg-base-100 rounded-box z-[1] mt-3 w-52 p-2 shadow border border-base-content/10"
                    >
                        <li>
                            <A
                                href="/postal-address"
                                on:click=move |_| {
                                    set_menu_open.set(false);
                                    blur_active_element();
                                }
                            >
                                "Postal Addresses"
                            </A>
                        </li>
                        <li>
                            <A
                                href="/"
                                on:click=move |_| {
                                    set_menu_open.set(false);
                                    blur_active_element();
                                }
                            >
                                "Sport Selection"
                            </A>
                        </li>
                    </ul>
                </div>
            </div>
        </header>
    }
}
