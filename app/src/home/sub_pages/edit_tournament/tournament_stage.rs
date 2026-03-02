//! Edit tournament stage component

#[cfg(feature = "test-mock")]
use app_utils::server_fn::stage::save_stage_inner;
use app_utils::{
    components::inputs::{InputCommitAction, NumberInput},
    hooks::{
        use_on_cancel::use_on_cancel,
        use_scroll_into_view::use_scroll_h2_into_view,
        use_url_navigation::{
            MatchedRouteHandler, UseMatchedRouteNavigationReturn, use_matched_route_navigation,
        },
    },
    params::{ParamQuery, StageNumberParams, TournamentBaseIdQuery},
    server_fn::stage::SaveStage,
    state::{
        EditorContextWithResource,
        object_table::ObjectEditorMapContext,
        tournament::{TournamentEditorContext, stage::StageEditorContext},
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn EditTournamentStage() -> impl IntoView {
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();
    let active_stage_number = StageNumberParams::use_param_query();

    // --- local state ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();

    let editor = Signal::derive(move || {
        if let Some(stage_number) = active_stage_number.get()
            && let Some(id) = tournament_base_id.get()
            && let Some(editor) = tournament_editor_map.get_editor(id)
            && let Some(stage_editor) = editor.get_stage_editor(stage_number)
        {
            Some((editor, stage_editor))
        } else {
            None
        }
    });

    view! {
        {move || {
            editor
                .get()
                .map(|(editor, stage_editor)| {
                    // if the editor for the current stage number exists in the map, render the stage editor form
                    view! {
                        <TournamentStageForm tournament_editor=editor stage_editor=stage_editor />
                        <div class="my-4"></div>
                        <Outlet />
                    }
                })
        }}
    }
}

#[component]
fn TournamentStageForm(
    tournament_editor: TournamentEditorContext,
    stage_editor: StageEditorContext,
) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseMatchedRouteNavigationReturn {
        url_matched_route,
        url_is_matched_route,
        ..
    } = use_matched_route_navigation();

    let active_stage_number = StageNumberParams::use_param_query();

    let editor_title = move || {
        if let Some(sn) = active_stage_number.get()
            && let Some(title) = tournament_editor
                .base_editor
                .mode
                .get()
                .and_then(|m| m.get_stage_name(sn))
        {
            format!("Edit {}", title)
        } else {
            "Edit Tournament Stage".to_string()
        }
    };

    // cancel function for close button
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    let on_submit = move || {
        if let Some(stage) = stage_editor.local.get()
            && let Some(base) = tournament_editor.base_editor.local.get()
            && stage.validate(&base).is_ok()
        {
            stage_editor.increment_optimistic_version();
            let data = SaveStage { stage };
            #[cfg(feature = "test-mock")]
            {
                let save_action = Action::new(|stage: &SaveStage| {
                    let stage = stage.clone();
                    async move {
                        let result = save_stage_inner(stage.stage).await;
                        leptos::web_sys::console::log_1(
                            &format!("Result of save stage: {:?}", result).into(),
                        );
                        result
                    }
                });
                save_action.dispatch(data);
            }
            #[cfg(not(feature = "test-mock"))]
            {
                stage_editor.save_stage.dispatch(data);
            }
        }
    };

    // For single stage or swiss system tournaments, ensure that stage 0 always has 1 group
    Effect::new(move || {
        if tournament_editor.base_editor.skip_stage_editor.get()
            && let Some(stage_number) = active_stage_number.get()
            && stage_number == 0
            && stage_editor.num_groups.get() != Some(1)
        {
            stage_editor.set_num_groups.run(Some(1));
            on_submit();
        }
    });

    view! {
        // hide stage editor for single stage and swiss system tournaments
        <Show when=move || !tournament_editor.base_editor.skip_stage_editor.get()>
            // Card wrapping Form and Group Links
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    <div class="flex justify-between items-center">
                        <h2 class="card-title" node_ref=scroll_ref data-testid="stage-editor-title">
                            {move || editor_title()}
                        </h2>
                        <button
                            class="btn btn-square btn-ghost btn-sm"
                            on:click=move |_| on_cancel.run(())
                            aria-label="Close"
                            data-testid="action-btn-close-edit-stage"
                        >
                            <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                        </button>
                    </div>
                    // --- Tournament Base Form ---
                    <div data-testid="tournament-editor-form">
                        <form on:submit:capture=move |ev| {
                            ev.prevent_default();
                            on_submit();
                        }>
                            <fieldset
                                disabled=move || { stage_editor.is_disabled_stage_editing.get() }
                                class="space-y-4 contents"
                                data-testid="stage-editor-form"
                            >
                                // hidden meta fields; can be used for E2E testing
                                <input
                                    type="hidden"
                                    data-testid="hidden-id"
                                    prop:value=move || {
                                        stage_editor.id.get().unwrap_or(Uuid::nil()).to_string()
                                    }
                                />
                                <input
                                    type="hidden"
                                    data-testid="hidden-version"
                                    prop:value=move || {
                                        stage_editor.version.get().unwrap_or_default().to_string()
                                    }
                                />
                                <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                    <NumberInput
                                        label="Number of Groups"
                                        name="stage-num-groups"
                                        data_testid="input-stage-num-groups"
                                        value=stage_editor.num_groups
                                        action=InputCommitAction::WriteAndSubmit(
                                            stage_editor.set_num_groups,
                                        )
                                        validation_result=stage_editor.validation_result
                                        min="1".to_string()
                                        object_id=stage_editor.id
                                        field="num_groups"
                                    />
                                </div>
                            // group editor links
                            </fieldset>
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                                <For
                                    each=move || {
                                        0..stage_editor.num_groups.get().unwrap_or_default()
                                    }
                                    key=|i| *i
                                    children=move |i| {
                                        view! {
                                            <button
                                                class="btn btn-sm btn-secondary"
                                                data-testid=format!("action-btn-configure-group-{}", i)
                                                on:click=move |_| {
                                                    let navigate = use_navigate();
                                                    tournament_editor
                                                        .prepare_group(
                                                            active_stage_number.get().unwrap_or_default(),
                                                            i,
                                                        );
                                                    let nav_url = url_matched_route(
                                                        MatchedRouteHandler::Extend(&i.to_string()),
                                                    );
                                                    navigate(
                                                        &nav_url,
                                                        NavigateOptions {
                                                            scroll: false,
                                                            ..Default::default()
                                                        },
                                                    );
                                                }
                                            >
                                                <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                {format!("Edit Group {}", i + 1)}
                                            </button>
                                        }
                                    }
                                />
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </Show>
    }
}
