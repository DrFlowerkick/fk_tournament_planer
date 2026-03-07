//! component to render a tree navigation for tournament objects
//! We use <details> and <summary> html elements to create a collapsible tree structure.
//! This allows us to easily show/hide child elements without needing complex state management
//! for the open/closed state of each node. We will use CSS to style the tree and indicate which nodes are expandable.

use crate::header::DropdownContext;
use app_core::CrTopic;
use app_utils::{
    error::{
        AppError, ComponentError,
        strategy::{handle_unexpected_ui_error, handle_with_error_banner},
    },
    hooks::{
        blur_active_element::blur_active_element,
        use_url_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{ParamQuery, TournamentBaseIdQuery},
    server_fn::stage::list_stage_ids_of_tournament,
    state::{
        SimpleEditorOptions,
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        object_table::ObjectEditorMapContext,
        toast_state::ToastContext,
        tournament::{TournamentEditorContext, stage::StageEditorContext},
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::prelude::*;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate};
use uuid::Uuid;

#[component]
pub fn TournamentTreeNavigation() -> impl IntoView {
    // --- state & context ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();

    let (editor, set_editor) = signal(None::<TournamentEditorContext>);

    // Use an effect to spawn the editor. This decouples the side-effect (creating a new context/resource)
    // from the view projection.
    Effect::new(move |_| {
        if let Some(id) = tournament_base_id.get() {
            // Only spawn if we have an ID
            let ed = tournament_editor_map
                .spawn_editor_for_edit_object(SimpleEditorOptions::with_id(id));
            set_editor.set(ed);
        } else {
            set_editor.set(None);
        }
    });

    view! {
        {move || {
            editor
                .get()
                .map(|ed| {
                    view! { <TournamentTreeBase tournament_editor=ed /> }
                })
        }}
    }
}

#[component]
fn TournamentTreeBase(tournament_editor: TournamentEditorContext) -> impl IntoView {
    // --- navigation hooks ---
    let UseQueryNavigationReturn {
        url_remove_query,
        url_update_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();

    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- state & context ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();
    let dropdown_ctx = expect_context::<DropdownContext>();

    // Resource that fetches stage ids for the tournament, to be used for rendering stage nodes in the tree
    let stage_ids = LocalResource::new(move || async move {
        if let Some(t_id) = tournament_base_id.try_get().flatten() {
            activity_tracker
                .track_activity_wrapper(
                    component_id.get_value(),
                    list_stage_ids_of_tournament(t_id),
                )
                .await
        } else {
            Ok(vec![])
        }
    });

    // Refetch callbacks
    let refetch = Callback::new(move |()| stage_ids.refetch());
    page_err_ctx.register_retry_handler(component_id.get_value(), refetch);

    let reload_after_new = Callback::new(move |()| {
        // in menu it should not be a problem to trigger a refetch while editing,
        // because it should not trigger rerendering of the currently edited stage,
        // but only update the list of stages in the tree, which should not interrupt
        // the editing process.
        toast_ctx.success("New Tournament Stage on server, adding stage to menu", None);
        stage_ids.refetch()
    });

    // Subscribe to relevant events from client registry to trigger refetch
    Effect::new(move || {
        if let Some(tournament_base_id) = tournament_editor.base_editor.id.get() {
            let topic = CrTopic::NewStage { tournament_base_id };
            use_client_registry_socket(topic.into(), None.into(), reload_after_new.clone());
        }
    });

    let on_cancel = Callback::new(move |()| {
        tournament_editor_map.remove_all();
        toast_ctx.info("Initializing...", None);
        let nav_url = url_remove_query(TournamentBaseIdQuery::KEY, Some("/"));
        navigate(
            &nav_url,
            NavigateOptions {
                scroll: false,
                ..Default::default()
            },
        );
    });

    view! {
        <Transition fallback=move || {
            view! { <span class="loading loading-spinner loading-xs"></span> }
        }>
            <ErrorBoundary fallback=move |errors| {
                for (_err_id, err) in errors.get().into_iter() {
                    let e = err.into_inner();
                    if let Some(comp_err) = e.downcast_ref::<ComponentError>() {
                        if let AppError::ResourceNotFound(object, _) = &comp_err.app_error
                            && object == "Tournament Base"
                        {
                            toast_ctx.error("Obsolete tournament id in query.", None);
                        } else {
                            handle_with_error_banner(&page_err_ctx, comp_err, on_cancel);
                        }
                    } else {
                        handle_unexpected_ui_error(
                            &page_err_ctx,
                            component_id.get_value(),
                            "An unexpected error occurred.",
                            on_cancel,
                        );
                    }
                }
            }>
                {move || {
                    tournament_editor
                        .base_editor
                        .load_tournament_base
                        .and_then(|maybe_base| {
                            maybe_base
                                .as_ref()
                                .map(|base| {
                                    tournament_editor.update_base_in_editor(base);
                                    view! {
                                        <details class="collapse border-base-300 border">
                                            <summary class="collapse-title font-semibold">
                                                <span class="icon-[heroicons--trophy] w-[1em] h-[1em] mr-2"></span>
                                                {move || tournament_editor.base_editor.name.get()}
                                                <A
                                                    href=move || {
                                                        if let Some(id) = tournament_editor
                                                            .base_editor
                                                            .id
                                                            .try_get()
                                                            .flatten()
                                                        {
                                                            url_update_query(
                                                                TournamentBaseIdQuery::KEY,
                                                                &id.to_string(),
                                                                Some("/tournaments/edit"),
                                                            )
                                                        } else {
                                                            "/tournaments/edit".to_string()
                                                        }
                                                    }
                                                    attr:class="btn btn-ghost btn-sm"
                                                    scroll=false
                                                    on:click=move |_| {
                                                        dropdown_ctx.set_menu_open.set(false);
                                                        blur_active_element();
                                                    }
                                                >
                                                    <span class="icon-[heroicons--link] w-[1em] h-[1em]"></span>
                                                </A>
                                            </summary>
                                            <div class="collapse-content text-sm">
                                                {move || {
                                                    stage_ids
                                                        .and_then(|s_ids_and_numbers| {
                                                            let s_ids_and_numbers = s_ids_and_numbers.clone();
                                                            {
                                                                view! {
                                                                    <For
                                                                        each=move || {
                                                                            s_ids_and_numbers
                                                                                .clone()
                                                                                .into_iter()
                                                                                .filter_map(move |(stage_id, stage_number)| {
                                                                                    tournament_editor
                                                                                        .spawn_stage_editor(Some(stage_id), stage_number)
                                                                                        .map(|stage_editor| (stage_id, stage_editor))
                                                                                })
                                                                        }
                                                                        key=|(stage_id, _)| *stage_id
                                                                        children=move |(_, stage_editor)| {
                                                                            view! {
                                                                                <div class="collapse-content text-sm">
                                                                                    <TournamentTreeStage
                                                                                        tournament_editor=tournament_editor
                                                                                        stage_editor=stage_editor
                                                                                    />
                                                                                </div>
                                                                            }
                                                                        }
                                                                    />
                                                                }
                                                            }
                                                        })
                                                }}
                                            </div>
                                        </details>
                                    }
                                })
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn TournamentTreeStage(
    tournament_editor: TournamentEditorContext,
    stage_editor: StageEditorContext,
) -> impl IntoView {
    // --- navigation hooks ---
    let UseQueryNavigationReturn {
        url_update_path, ..
    } = use_query_navigation();
    let dropdown_ctx = expect_context::<DropdownContext>();

    view! {
        {move || {
            stage_editor
                .load_stage
                .and_then(|maybe_stage| {
                    maybe_stage
                        .as_ref()
                        .map(|stage| {
                            tournament_editor.update_stage_in_editor(stage);
                            view! {
                                <details class="collapse border-base-300 border">
                                    <summary class="collapse-title font-semibold">
                                        <span class="icon-[heroicons--view-columns-16-solid] w-[1em] h-[1em] mr-2"></span>
                                        {move || {
                                            stage_editor
                                                .number
                                                .get()
                                                .and_then(|stage_number| {
                                                    tournament_editor
                                                        .base_editor
                                                        .mode
                                                        .get()
                                                        .and_then(|mode| mode.get_stage_name(stage_number))
                                                })
                                        }}
                                        <A
                                            href=move || {
                                                if let Some(stage_number) = stage_editor.number.get() {
                                                    url_update_path(
                                                        &format!("/tournaments/edit/{}", stage_number),
                                                    )
                                                } else {
                                                    "/tournaments/edit".to_string()
                                                }
                                            }
                                            attr:class="btn btn-ghost btn-sm"
                                            scroll=false
                                            on:click=move |_| {
                                                dropdown_ctx.set_menu_open.set(false);
                                                blur_active_element();
                                            }
                                        >
                                            <span class="icon-[heroicons--link] w-[1em] h-[1em]"></span>
                                        </A>
                                    </summary>
                                </details>
                            }
                        })
                })
        }}
    }
}
