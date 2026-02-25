//! create or edit a tournament

use super::EditTournamentFallback;
use app_core::{TournamentBase, TournamentMode};
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, NumberInput, TextInput},
    enum_utils::EditAction,
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::{EditActionParams, FilterNameQuery, ParamQuery, TournamentBaseIdQuery},
    server_fn::tournament_base::SaveTournamentBase,
    state::{
        EditorContext, EditorContextWithResource, object_table::ObjectEditorMapContext,
        tournament::TournamentEditorContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

#[component]
pub fn EditTournamentBase() -> impl IntoView {
    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_is_matched_route,
        ..
    } = use_query_navigation();

    let edit_action = EditActionParams::use_param_query();
    let tournament_id = TournamentBaseIdQuery::use_param_query();

    // --- local state ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();

    let show_form = Signal::derive(move || {
        if let Some(id) = tournament_id.get()
            && let Some(editor) = tournament_editor_map.get_editor(id)
        {
            leptos::logging::debug_log!("Checking Edit action");
            match edit_action.get() {
                Some(EditAction::Edit) => editor
                    .origin_signal()
                    .try_with(|origin| origin.is_some())
                    .unwrap_or(false),
                Some(EditAction::New) => {
                    let show_form = editor
                        .origin_signal()
                        .try_with(|origin| origin.is_none())
                        .unwrap_or(false);
                    leptos::logging::debug_log!(
                        "New Tournament id: {}\nshow_form: {}",
                        id,
                        show_form
                    );
                    leptos::logging::debug_log!(
                        "Editor origin: {:?}",
                        editor.origin_signal().try_get()
                    );
                    leptos::logging::debug_log!(
                        "Editor local base: {:?}",
                        editor.base_editor.local.try_get()
                    );
                    leptos::logging::debug_log!(
                        "Editor origin base: {:?}",
                        editor.base_editor.origin_signal().try_get()
                    );
                    show_form
                }
                Some(EditAction::Copy) => editor
                    .origin_signal()
                    .try_with(|origin| origin.is_none())
                    .unwrap_or(false),
                None => false,
            }
        } else {
            false
        }
    });

    // remove unsaved editor (no origin) on unmount
    on_cleanup(move || {
        if let Some(id) = tournament_id.get_untracked()
            && let Some(editor) = tournament_editor_map.get_editor_untracked(id)
            && editor
                .origin_signal()
                .try_with(|origin| origin.is_none())
                .unwrap_or(false)
        {
            tournament_editor_map.remove_editor(id);
        }
    });

    // cancel function for close button
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Show when=move || edit_action.get().is_some() fallback=|| "Page not found.".into_view()>
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    <div class="flex justify-between items-center">
                        <h2 class="card-title" node_ref=scroll_ref>
                            {move || match edit_action.get() {
                                Some(EditAction::New) => "New Tournament",
                                Some(EditAction::Edit) => "Edit Tournament",
                                Some(EditAction::Copy) => "Copy Tournament",
                                None => "",
                            }}
                        </h2>
                        <button
                            class="btn btn-square btn-ghost btn-sm"
                            on:click=move |_| on_cancel.run(())
                            aria-label="Close"
                            data-testid="action-btn-close"
                        >
                            <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                        </button>
                    </div>
                    <Show
                        when=move || show_form.try_get().unwrap_or(false)
                        fallback=move || {
                            view! { <EditTournamentFallback /> }
                        }
                    >
                        // Using For forces the view to be recreated when the id changes
                        <For
                            each=move || {
                                tournament_id
                                    .get()
                                    .and_then(|current_id| {
                                        tournament_editor_map
                                            .get_editor(current_id)
                                            .map(|editor| (current_id, editor))
                                    })
                                    .into_iter()
                            }
                            key=|(id, _)| *id
                            children=move |(_, editor)| {
                                view! { <TournamentBaseForm tournament_editor=editor /> }
                            }
                        />
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn TournamentBaseForm(tournament_editor: TournamentEditorContext) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_matched_route,
        url_matched_route_update_queries,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let edit_action = EditActionParams::use_param_query();
    let intent = Signal::derive(move || {
        edit_action.get().map(|action| match action {
            EditAction::Edit => "update".to_string(),
            EditAction::New | EditAction::Copy => "create".to_string(),
        })
    });
    let show_stage_navigation =
        Signal::derive(move || matches!(edit_action.get(), Some(EditAction::Edit)));

    let post_save_callback = Callback::new(move |tb: TournamentBase| {
        if let Some(edit_action) = edit_action.get()
            && matches!(edit_action, EditAction::New | EditAction::Copy)
        {
            let tb_id = tb.get_id().to_string();
            let key_value = vec![
                (TournamentBaseIdQuery::KEY, tb_id.as_str()),
                (FilterNameQuery::KEY, tb.get_name()),
            ];
            // we need to use extend here, because the callback is executed in the route of
            // the list view
            let nav_url = url_matched_route_update_queries(
                key_value,
                MatchedRouteHandler::Extend(EditAction::Edit.to_string().as_str()),
            );
            // ToDo: remove this, if nav_url is ok after testing
            leptos::logging::debug_log!("Navigating to {} after saving tournament base", nav_url);
            navigate(
                &nav_url,
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });
    tournament_editor
        .base_editor
        .post_save_callback
        .set_value(Some(post_save_callback));

    view! {
        // --- Tournament Base Form ---
        <div data-testid="tournament-editor-form">
            <ActionForm
                action=tournament_editor.base_editor.save_tournament_base
                on:submit:capture=move |ev| {
                    ev.prevent_default();
                    if tournament_editor.base_editor.validation_result.with(|vr| vr.is_err()) {
                        return;
                    }
                    if let Some(base) = tournament_editor.base_editor.local.get() {
                        tournament_editor.base_editor.increment_optimistic_version();
                        let save_base = SaveTournamentBase { base };
                        tournament_editor.base_editor.save_tournament_base.dispatch(save_base);
                    }
                }
            >
                // --- Tournament Base Form ---
                <fieldset
                    class="space-y-4 contents"
                    disabled=move || {
                        tournament_editor.base_editor.is_disabled_base_editing.get()
                    }
                >
                    // hidden meta fields; can be used for E2E testing
                    <input
                        type="hidden"
                        name="tournament[id]"
                        data-testid="hidden-id"
                        prop:value=move || {
                            tournament_editor
                                .base_editor
                                .id
                                .get()
                                .unwrap_or(Uuid::nil())
                                .to_string()
                        }
                    />
                    <input
                        type="hidden"
                        name="tournament[version]"
                        data-testid="hidden-version"
                        prop:value=move || {
                            tournament_editor.base_editor.version.get().unwrap_or_default()
                        }
                    />
                    <input
                        type="hidden"
                        name="tournament[sport_id]"
                        data-testid="hidden-sport-id"
                        prop:value=move || {
                            tournament_editor
                                .base_editor
                                .sport_id
                                .get()
                                .unwrap_or(Uuid::nil())
                                .to_string()
                        }
                    />
                    <input
                        type="hidden"
                        name="tournament[t_type]"
                        data-testid="hidden-t_type"
                        prop:value=move || {
                            tournament_editor
                                .base_editor
                                .tournament_type
                                .get()
                                .map(|tt| tt.to_string())
                                .unwrap_or_default()
                        }
                    />
                    <input
                        type="hidden"
                        name="tournament[state]"
                        data-testid="hidden-state"
                        prop:value=move || {
                            tournament_editor
                                .base_editor
                                .tournament_state
                                .get()
                                .map(|ts| ts.to_string())
                                .unwrap_or_default()
                        }
                    />
                    <input
                        type="hidden"
                        name="intent"
                        data-testid="intent"
                        prop:value=move || intent.get()
                    />
                    <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                        <TextInput
                            label="Tournament Name"
                            name="tournament-name"
                            data_testid="input-tournament-name"
                            value=tournament_editor.base_editor.name
                            action=InputCommitAction::WriteAndSubmit(
                                tournament_editor.base_editor.set_name,
                            )
                            validation_result=tournament_editor.base_editor.validation_result
                            object_id=tournament_editor.base_editor.id
                            field="name"
                        />

                        <NumberInput
                            label="Number of Entrants"
                            name="tournament-entrants"
                            data_testid="input-tournament-entrants"
                            value=tournament_editor.base_editor.num_entrants
                            action=InputCommitAction::WriteAndSubmit(
                                tournament_editor.base_editor.set_num_entrants,
                            )
                            validation_result=tournament_editor.base_editor.validation_result
                            object_id=tournament_editor.base_editor.id
                            field="num_entrants"
                            min="2".to_string()
                        />

                        <EnumSelect
                            label="Mode"
                            name="tournament-mode"
                            data_testid="select-tournament-mode"
                            value=tournament_editor.base_editor.mode
                            action=InputCommitAction::WriteAndSubmit(
                                tournament_editor.base_editor.set_mode,
                            )
                        />

                        <Show when=move || {
                            matches!(
                                tournament_editor.base_editor.mode.get(),
                                Some(TournamentMode::SwissSystem { .. })
                            )
                        }>
                            <NumberInput
                                label="Rounds (Swiss System)"
                                name="tournament-swiss-num_rounds"
                                data_testid="input-tournament-swiss-num_rounds"
                                value=tournament_editor.base_editor.num_rounds_swiss_system
                                action=InputCommitAction::WriteAndSubmit(
                                    tournament_editor.base_editor.set_num_rounds_swiss_system,
                                )
                                validation_result=tournament_editor.base_editor.validation_result
                                object_id=tournament_editor.base_editor.id
                                field="mode.num_rounds"
                                min="1".to_string()
                            />
                        </Show>

                    </div>
                </fieldset>
                <Show when=move || show_stage_navigation.get()>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                        <For
                            each=move || {
                                0..tournament_editor
                                    .base_editor
                                    .mode
                                    .get()
                                    .map(|m| m.get_num_of_stages())
                                    .unwrap_or_default()
                            }
                            key=|i| *i
                            children=move |i| {
                                let stage_name = move || {
                                    tournament_editor
                                        .base_editor
                                        .mode
                                        .get()
                                        .and_then(|m| m.get_stage_name(i))
                                        .unwrap_or_else(|| format!("Stage {}", i + 1))
                                };
                                view! {
                                    <button
                                        class="btn btn-sm"
                                        class:btn-primary=move || {
                                            !tournament_editor.base_editor.skip_stage_editor.get()
                                        }
                                        class:btn-secondary=move || {
                                            tournament_editor.base_editor.skip_stage_editor.get()
                                        }
                                        data-testid=move || {
                                            format!(
                                                "action-btn-configure-stage-{}",
                                                stage_name().to_lowercase().replace(" ", "-"),
                                            )
                                        }
                                        on:click=move |_| {
                                            let navigate = use_navigate();
                                            let path = if tournament_editor
                                                .base_editor
                                                .skip_stage_editor
                                                .get()
                                            {
                                                tournament_editor.prepare_group(0, 0);
                                                "0/0".to_string()
                                            } else {
                                                tournament_editor.prepare_stage(i);
                                                i.to_string()
                                            };
                                            let nav_url = url_matched_route(
                                                MatchedRouteHandler::Extend(&path),
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
                                        {move || format!("Edit {}", stage_name())}
                                    </button>
                                }
                            }
                        />
                    </div>
                </Show>
            </ActionForm>
        </div>
    }
}
