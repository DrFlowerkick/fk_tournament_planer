//! create or edit a tournament

use app_core::TournamentMode;
use app_utils::{
    components::inputs::{
        EnumSelectWithValidation, NumberInputWithValidation, TextInputWithValidation,
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    state::{error_state::PageErrorContext, tournament_editor::TournamentEditorContext},
};
use leptos::prelude::*;
use leptos_router::{components::A, nested_router::Outlet};

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Initialize context for creating and editing tournaments ---
    let tournament_editor_context = TournamentEditorContext::new();
    provide_context(tournament_editor_context);
    let page_err_ctx = expect_context::<PageErrorContext>();

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_route_with_sub_path,
        ..
    } = use_query_navigation();

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    view! {
        <div
            class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
            data-testid="tournament-editor-root"
        >
            <div class="w-full flex justify-between items-center pb-4">
                <h2 class="text-3xl font-bold" data-testid="tournament-editor-title">
                    {move || {
                        if tournament_editor_context.is_new_tournament.get() {
                            "Plan New Tournament"
                        } else {
                            "Edit Tournament"
                        }
                    }}
                </h2>
            </div>
            <Show
                when=move || {
                    tournament_editor_context.is_new_tournament.get()
                        || tournament_editor_context.tournament_id.get().is_some()
                }
                fallback=|| {
                    view! {
                        <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                            <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                            <p class="text-2xl font-bold text-center">
                                "Please select a tournament from the list."
                            </p>
                        </div>
                    }
                }
            >
                // Card wrapping Form and Stage Links
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        // --- Form Area ---
                        <fieldset
                            disabled=move || {
                                page_err_ctx.has_errors()
                                    || tournament_editor_context.is_disabled_base_editing.get()
                                    || tournament_editor_context.is_busy.get()
                                    || !tournament_editor_context.is_base_initialized.get()
                            }
                            class="contents"
                            data-testid="tournament-editor-form"
                        >
                            <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                                <TextInputWithValidation
                                    label="Tournament Name"
                                    name="tournament-name"
                                    value=tournament_editor_context.base_name
                                    set_value=tournament_editor_context.set_base_name
                                    validation_result=tournament_editor_context.validation_result
                                    object_id=tournament_editor_context.base_id
                                    field="name"
                                />

                                <NumberInputWithValidation
                                    label="Number of Entrants"
                                    name="tournament-entrants"
                                    value=tournament_editor_context.base_num_entrants
                                    set_value=tournament_editor_context.set_base_num_entrants
                                    validation_result=tournament_editor_context.validation_result
                                    object_id=tournament_editor_context.base_id
                                    field="num_entrants"
                                    min="2".to_string()
                                />

                                <EnumSelectWithValidation
                                    label="Mode"
                                    name="tournament-mode"
                                    value=tournament_editor_context.base_mode
                                    set_value=tournament_editor_context.set_base_mode
                                    validation_result=tournament_editor_context.validation_result
                                    object_id=None
                                    field="No Direct Validation"
                                />

                                <Show when=move || {
                                    matches!(
                                        tournament_editor_context.base_mode.get(),
                                        Some(TournamentMode::SwissSystem { .. })
                                    )
                                }>
                                    <NumberInputWithValidation
                                        label="Rounds (Swiss System)"
                                        name="tournament-swiss-num_rounds"
                                        value=tournament_editor_context.base_num_rounds_swiss_system
                                        set_value=tournament_editor_context
                                            .set_base_num_rounds_swiss_system
                                        validation_result=tournament_editor_context
                                            .validation_result
                                        object_id=tournament_editor_context.base_id
                                        field="mode.num_rounds"
                                        min="1".to_string()
                                    />
                                </Show>

                            </div>
                        // stages editor links
                        </fieldset>
                        {move || match tournament_editor_context.base_mode.get() {
                            Some(TournamentMode::SingleStage) => {
                                // set up single stage editor
                                // with single stage we only have one group in stage
                                // therefore we "skip" the stage editor and go directly to group editor
                                view! {
                                    <div class="w-full mt-6">
                                        <A
                                            href=move || url_route_with_sub_path("0/0")
                                            attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-single-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                            "Edit Single Stage"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(TournamentMode::PoolAndFinalStage) => {
                                // set up pool and final stage editor
                                // with pool and final stage we have two stages to configure
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full mt-6">
                                        <A
                                            href=move || url_route_with_sub_path("0")
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_route_with_sub_path("1")
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-final-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Final Stage"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(TournamentMode::TwoPoolStagesAndFinalStage) => {
                                // set up two pool stages and final stage editor
                                // with pool and final stage we have three stages to configure
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                                        <A
                                            href=move || url_route_with_sub_path("0")
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-first-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit First Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_route_with_sub_path("1")
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-second-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Second Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_route_with_sub_path("2")
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-final-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Final Stage"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(TournamentMode::SwissSystem { .. }) => {
                                // set up swiss system stage editor
                                // with swiss system we only have one group in stage
                                // therefore we "skip" the stage editor and go directly to group editor
                                view! {
                                    <div class="w-full mt-6">
                                        <A
                                            href=move || url_route_with_sub_path("0/0")
                                            attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-swiss-system"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                            "Edit Swiss System"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                            None => ().into_any(),
                        }}
                    </div>
                </div>
            </Show>

            <Outlet />

            <Show when=move || {
                tournament_editor_context.is_new_tournament.get()
                    || tournament_editor_context.tournament_id.get().is_some()
            }>
                // --- Action Buttons ---
                <div class="w-full flex justify-end gap-4 pt-6">
                    <button
                        class="btn btn-ghost"
                        data-testid="btn-tournament-cancel"
                        on:click=move |_| on_cancel.run(())
                        disabled=move || tournament_editor_context.is_busy.get()
                    >
                        "Cancel"
                    </button>

                    <button
                        class="btn btn-primary"
                        data-testid="btn-tournament-save"
                        on:click=move |_| tournament_editor_context.save_diff()
                        disabled=move || {
                            !tournament_editor_context.is_changed.get()
                                || tournament_editor_context.validation_result.get().is_err()
                                || tournament_editor_context.is_busy.get()
                                || page_err_ctx.has_errors()
                        }
                    >
                        <Show
                            when=move || !tournament_editor_context.is_busy.get()
                            fallback=|| {
                                view! { <span class="loading loading-spinner">"Saving..."</span> }
                            }
                        >
                            <span>"Save Tournament"</span>
                        </Show>
                    </button>
                </div>
            </Show>
        </div>
    }
}
