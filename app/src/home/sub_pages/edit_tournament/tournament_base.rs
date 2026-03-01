//! create or edit a tournament

use app_core::{TournamentBase, TournamentMode};
#[cfg(feature = "test-mock")]
use app_utils::server_fn::tournament_base::save_tournament_base_inner;
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, NumberInput, TextInput},
    enum_utils::EditAction,
    hooks::{
        use_on_cancel::use_on_cancel,
        use_scroll_into_view::use_scroll_h2_into_view,
        use_url_navigation::{
            MatchedRouteHandler, UseMatchedRouteNavigationReturn, UseQueryNavigationReturn,
            use_matched_route_navigation, use_query_navigation,
        },
    },
    params::{EditActionParams, FilterNameQuery, ParamQuery, TournamentBaseIdQuery},
    server_fn::tournament_base::SaveTournamentBase,
    state::{
        EditorContext, EditorContextWithResource, object_table::ObjectEditorMapContext,
        tournament::TournamentEditorContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn EditTournamentBase() -> impl IntoView {
    // --- Hooks & Navigation ---
    let UseMatchedRouteNavigationReturn {
        url_is_matched_route,
        ..
    } = use_matched_route_navigation();

    let edit_action = EditActionParams::use_param_query();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();

    // --- local state ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();

    // remove unsaved editor (no origin) on unmount
    on_cleanup(move || {
        if let Some(id) = tournament_base_id.get_untracked()
            && let Some(editor) = tournament_editor_map.get_editor_untracked(id)
            && editor
                .origin_signal()
                .with_untracked(|origin| origin.is_none())
        {
            tournament_editor_map.remove_editor(id);
        }
    });

    let editor = Signal::derive(move || {
        if let Some(id) = tournament_base_id.get()
            && let Some(editor) = tournament_editor_map.get_editor(id)
            && match edit_action.get() {
                Some(EditAction::Edit) => editor.origin_signal().with(|origin| origin.is_some()),
                Some(EditAction::New) => editor.origin_signal().with(|origin| origin.is_none()),
                Some(EditAction::Copy) => editor.origin_signal().with(|origin| origin.is_none()),
                None => false,
            }
        {
            Some(editor)
        } else {
            None
        }
    });

    // cancel function for close button
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Show when=move || edit_action.get().is_some() fallback=|| "Page not found.".into_view()>
            <div class="card w-full bg-base-100 shadow-xl" data-testid="tournament-editor-root">
                <div class="card-body">
                    <div class="flex justify-between items-center">
                        <h2
                            class="card-title"
                            node_ref=scroll_ref
                            data-testid="tournament-editor-title"
                        >
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
                            data-testid="action-btn-close-edit-base"
                        >
                            <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                        </button>
                    </div>
                    {move || {
                        editor
                            .try_get()
                            .flatten()
                            .map(|ed| {
                                view! { <TournamentBaseForm tournament_editor=ed /> }.into_any()
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                        <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                        <p class="text-2xl font-bold text-center">
                                            {move || match edit_action.try_get().flatten() {
                                                Some(EditAction::New) => {
                                                    "Press 'New Tournament' to create a new tournament."
                                                }
                                                Some(EditAction::Edit) => {
                                                    "Please select a tournament from the list."
                                                }
                                                Some(EditAction::Copy) => {
                                                    "Press 'Copy selected Tournament' to create a new tournament based upon the selected one."
                                                }
                                                None => "",
                                            }}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            })
                    }}
                </div>
            </div>
            <div class="my-4"></div>
            <Outlet />
        </Show>
    }
}

#[component]
fn TournamentBaseForm(tournament_editor: TournamentEditorContext) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_update_queries, ..
    } = use_query_navigation();
    let UseMatchedRouteNavigationReturn {
        url_matched_route, ..
    } = use_matched_route_navigation();
    let navigate = use_navigate();

    let edit_action = EditActionParams::use_param_query();
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
            let nav_url = url_update_queries(key_value, Some("/tournaments/edit"));
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

    let on_submit = move || {
        if let Some(base) = tournament_editor.base_editor.local.get()
            && base.validate().is_ok()
        {
            tournament_editor.base_editor.increment_optimistic_version();
            let data = SaveTournamentBase { base };
            #[cfg(feature = "test-mock")]
            {
                let save_action = Action::new(|base: &SaveTournamentBase| {
                    let base = base.clone();
                    async move {
                        let result = save_tournament_base_inner(base.base).await;
                        leptos::web_sys::console::log_1(
                            &format!("Result of save tournament base: {:?}", result).into(),
                        );
                        result
                    }
                });
                save_action.dispatch(data);
            }
            #[cfg(not(feature = "test-mock"))]
            {
                tournament_editor
                    .base_editor
                    .save_tournament_base
                    .dispatch(data);
            }
        }
    };

    view! {
        // --- Tournament Base Form ---
        <div data-testid="tournament-editor-form">
            <form on:submit:capture=move |ev| {
                ev.prevent_default();
                on_submit();
            }>
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
                        data-testid="hidden-version"
                        prop:value=move || {
                            tournament_editor
                                .base_editor
                                .version
                                .get()
                                .unwrap_or_default()
                                .to_string()
                        }
                    />
                    <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                        <TextInput
                            label="Tournament Name"
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
                                            tournament_editor.prepare_stage(i);
                                            let path = if tournament_editor
                                                .base_editor
                                                .skip_stage_editor
                                                .get()
                                            {
                                                tournament_editor.prepare_group(0, 0);
                                                "0/0".to_string()
                                            } else {
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
            </form>
        </div>
    }
}
