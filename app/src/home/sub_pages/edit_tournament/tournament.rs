//! component to edit a tournament

use super::{EditTournamentBase, EditTournamentFallback};
use app_utils::{
    params::{ParamQuery, TournamentBaseIdQuery},
    state::{
        EditorContext, object_table::ObjectEditorMapContext, tournament::TournamentEditorContext,
    },
};
use leptos::prelude::*;
use leptos_router::nested_router::Outlet;

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
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

    view! {
        <Show
            when=move || {
                tournament_base_id
                    .try_get()
                    .flatten()
                    .and_then(|id| tournament_editor_map.get_editor(id))
                    .is_some()
            }
            fallback=|| {
                view! {
                    <div class="card w-full bg-base-100 shadow-xl">
                        <div class="card-body">
                            <EditTournamentFallback />
                        </div>
                    </div>
                }
            }
        >
            // Using For forces the view to be recreated when the id changes
            <For
                each=move || {
                    tournament_base_id
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
                    view! {
                        // ToDo: we probably need some html structure here
                        <TournamentMenu tournament_editor=editor />
                        <EditTournamentBase />
                        <div class="my-4"></div>
                        <Outlet />
                    }
                }
            />
        </Show>
    }
}

#[component]
fn TournamentMenu(tournament_editor: TournamentEditorContext) -> impl IntoView {}
