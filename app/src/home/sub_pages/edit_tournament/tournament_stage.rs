//! Edit tournament stage component

use app_utils::{
    components::inputs::NumberInputWithValidation,
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    state::{error_state::PageErrorContext, tournament_editor::TournamentEditorContext},
};
use leptos::prelude::*;
use leptos_router::{components::A, nested_router::Outlet};

#[component]
pub fn EditTournamentStage() -> impl IntoView {
    // --- Get context for creating and editing tournaments ---
    let tournament_editor_context = expect_context::<TournamentEditorContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_route_with_sub_path,
        ..
    } = use_query_navigation();

    let editor_title = move || {
        if let Some(sn) = tournament_editor_context.active_stage_number.get()
            && let Some(title) = tournament_editor_context
                .base_mode
                .get()
                .and_then(|m| m.get_stage_name(sn))
        {
            format!("Edit {}", title)
        } else {
            "Edit Tournament Stage".to_string()
        }
    };

    view! {
        // hide stage editor for single stage and swiss system tournaments
        <Show when=move || !tournament_editor_context.is_hiding_stage_editor.get()>
            // Show loading spinner while tournament is being loaded
            <Show
                when=move || tournament_editor_context.is_stage_initialized.get()
                fallback=move || {
                    view! {
                        <div class="w-full flex justify-center py-8">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    }
                }
            >
                // Card wrapping Form and Group Links
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        // --- Form Area ---
                        <div
                            class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
                            data-testid="stage-editor-root"
                        >
                            <div class="w-full flex justify-between items-center pb-4">
                                <h2 class="text-3xl font-bold" data-testid="stage-editor-title">
                                    {move || editor_title()}
                                </h2>
                            </div>
                            <fieldset
                                disabled=move || {
                                    tournament_editor_context.is_disabled_stage_editing.get()
                                        || tournament_editor_context.is_busy.get()
                                        || page_err_ctx.has_errors()
                                }
                                class="contents"
                                data-testid="stage-editor-form"
                            >
                                <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                    <NumberInputWithValidation
                                        label="Number of Groups"
                                        name="stage-num-groups"
                                        value=tournament_editor_context.stage_num_groups
                                        set_value=tournament_editor_context.set_stage_num_groups
                                        validation_result=tournament_editor_context
                                            .validation_result
                                        min="1".to_string()
                                        object_id=tournament_editor_context.active_stage_id
                                        field="num_groups"
                                    />
                                </div>
                            // group editor links
                            </fieldset>
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                                <For
                                    each=move || {
                                        0..tournament_editor_context
                                            .stage_num_groups
                                            .get()
                                            .unwrap_or_default()
                                    }
                                    key=|i| *i
                                    children=move |i| {
                                        view! {
                                            <A
                                                href=move || url_route_with_sub_path(&i.to_string())
                                                attr:class="btn btn-secondary h-auto min-h-[4rem] text-lg shadow-md"
                                                attr:data-testid=format!("link-configure-group-{}", i)
                                                scroll=false
                                            >
                                                <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                {format!("Edit Group {}", i + 1)}
                                            </A>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </Show>
        <Outlet />
    }
}
