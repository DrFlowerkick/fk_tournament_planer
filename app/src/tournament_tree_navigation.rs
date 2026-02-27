//! component to render a tree navigation for tournament objects
//! We use <details> and <summary> html elements to create a collapsible tree structure.
//! This allows us to easily show/hide child elements without needing complex state management
//! for the open/closed state of each node. We will use CSS to style the tree and indicate which nodes are expandable.

use app_utils::{
    params::{ParamQuery, TournamentBaseIdQuery},
    server_fn::stage::list_stage_ids_of_tournament,
    state::{
        SimpleEditorOptions, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table::ObjectEditorMapContext, toast_state::ToastContext,
        tournament::TournamentEditorContext,
    },
};
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn TournamentTreeNavigation() -> impl IntoView {
    // --- state & context ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();

    view! {
        // Using For forces the view to be recreated when the id changes
        <For
            each=move || {
                tournament_base_id
                    .try_get()
                    .flatten()
                    .and_then(|current_id| {
                        tournament_editor_map
                            .spawn_editor_for_edit_object(SimpleEditorOptions::with_id(current_id))
                            .map(|editor| (current_id, editor))
                    })
                    .into_iter()
            }
            key=|(id, _)| *id
            children=move |(_, editor)| {
                view! { <TournamentTreeBase tournament_editor=editor /> }
            }
        />
    }
}

#[component]
fn TournamentTreeBase(tournament_editor: TournamentEditorContext) -> impl IntoView {
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

    // Resource that fetches stage ids for the tournament, to be used for rendering stage nodes in the tree
    let stage_ids = Resource::new(
        move || tournament_editor.base_editor.id.get(),
        move |maybe_id| async move {
            if let Some(t_id) = maybe_id {
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        list_stage_ids_of_tournament(t_id),
                    )
                    .await
            } else {
                Ok(vec![])
            }
        },
    );
}
