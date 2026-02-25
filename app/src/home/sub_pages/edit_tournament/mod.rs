//! Edit tournament components

pub mod tournament;
pub mod tournament_base;
pub mod tournament_group;
pub mod tournament_stage;

pub use tournament::*;
pub use tournament_base::*;
pub use tournament_group::*;
pub use tournament_stage::*;

use app_utils::{
    enum_utils::EditAction,
    params::{EditActionParams, GroupNumberParams, ParamQuery, StageNumberParams},
};
use leptos::prelude::*;
#[allow(unused_imports)]
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    ParamSegment,
    any_nested_route::IntoAnyNestedRoute,
    components::{ParentRoute, Route},
    path,
};

#[component]
fn EditTournamentFallback() -> impl IntoView {
    let edit_action = EditActionParams::use_param_query();

    view! {
        <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
            <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
            <p class="text-2xl font-bold text-center">
                {move || match edit_action.try_get().flatten() {
                    Some(EditAction::New) => "Press 'New Tournament' to create a new tournament.",
                    Some(EditAction::Edit) => "Please select a tournament from the list.",
                    Some(EditAction::Copy) => {
                        "Press 'Copy selected Tournament' to create a new tournament based upon the selected one."
                    }
                    None => "",
                }}
            </p>
        </div>
    }
}

#[component(transparent)]
pub fn EditSubRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=ParamSegment(StageNumberParams::KEY) view=EditTournamentStage>
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
            <Route path=ParamSegment(GroupNumberParams::KEY) view=EditTournamentGroup />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
