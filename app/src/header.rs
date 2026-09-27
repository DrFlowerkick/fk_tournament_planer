//! header component

use crate::{
    home::select_sport::STORAGE_KEY_SPORT_ID, tournament_tree_navigation::TournamentTreeNavigation,
};
use app_utils::{
    hooks::{
        blur_active_element::blur_active_element,
        use_url_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{ParamQuery, TournamentBaseIdQuery},
    state::{
        activity_tracker::ActivityTracker, object_table::ObjectEditorMapContext,
        tournament::TournamentEditorContext,
    },
};
use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone, Copy)]
pub struct DropdownContext {
    pub set_menu_open: WriteSignal<bool>,
}

#[component]
pub fn Header() -> impl IntoView {
    // navigation hooks
    let UseQueryNavigationReturn {
        url_update_path, ..
    } = use_query_navigation();

    // Get the activity tracker context to reactively toggle the inert state
    let activity_tracker = expect_context::<ActivityTracker>();

    // Signal to manage the mobile menu state
    let (menu_open, set_menu_open) = signal(false);
    let dropdown_ctx = DropdownContext { set_menu_open };
    provide_context(dropdown_ctx);

    // prepare tournament editor context
    // This is used for menu navigation of selected tournament and at the same time loads
    // all objects of a tournament into context, which may be used by the editor.
    let tournament_editor_map =
        ObjectEditorMapContext::<TournamentEditorContext, TournamentBaseIdQuery>::new();
    provide_context(tournament_editor_map);

    let tournament_base_id = TournamentBaseIdQuery::use_param_query();

    view! {
        <header class="navbar bg-base-300 sticky top-0 z-50">
            <div class="flex-1">
                <A href=move || url_update_path("/") attr:class="btn btn-ghost normal-case text-xl">
                    "Tournament Planner"
                </A>
            </div>
            // Group loading indicator and menu button together on the right
            <div class="flex-none flex items-center gap-3 px-2">
                <Show when=move || activity_tracker.is_active.get()>
                    <span class="loading loading-bars loading-sm"></span>
                </Show>
                <div class="dropdown dropdown-end" class:dropdown-open=move || menu_open.get()>
                    // Use a button instead of label/input to avoid event conflicts in Leptos.
                    // The 'swap-active' class controls which icon is visible based on the signal.
                    // Trigger blur if false or when clicking links to ensure closing the dropdown menu.
                    // This is required because daisyUI's dropdown relies on focus/blur of CSS selectors.
                    <button
                        type="button"
                        data-testid="menu-button"
                        class="btn btn-ghost btn-circle swap swap-rotate"
                        class:swap-active=move || menu_open.get()
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
                        class="dropdown-content menu bg-base-100 rounded-box z-[1] mt-3 w-max max-w-sm p-2 shadow border border-base-content/10"
                    >
                        <li>
                            <A
                                href=move || url_update_path("/postal-address")
                                attr:data-testid="nav-postal-addresses"
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
                                attr:data-testid="nav-sport-selection"
                                on:click=move |_| {
                                    if let Ok(Some(storage)) = window().local_storage() {
                                        let _ = storage.remove_item(STORAGE_KEY_SPORT_ID);
                                    }
                                    set_menu_open.set(false);
                                    blur_active_element();
                                }
                            >
                                "Sport Selection"
                            </A>
                        </li>
                        <Show when=move || tournament_base_id.get().is_some()>
                            <li class="menu-title border-t border-base-content/10 my-1 py-0 h-px"></li>
                            <li>
                                <TournamentTreeNavigation />
                            </li>
                        </Show>
                    </ul>
                </div>
            </div>
        </header>
    }
}
