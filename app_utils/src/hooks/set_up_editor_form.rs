//! generic setup hook for all editors

use crate::{
    enum_utils::EditAction,
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{EditActionParams, ParamQuery, ParamQueryId},
    state::EditorContext,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};

/// Generic setup for editor forms that handles the logic of showing the form based on the
/// matched route and the loaded object in the editor context. The function returns a signal
/// that indicates whether the form should be shown or not. The logic is as follows:
///
/// EditAction::Edit: show form if an existing object is loaded in the editor context.
/// EditAction::Copy:
/// 1.) If an object id is given in query, navigate to edit with the given id.
/// 2.) Else if an existing object is loaded in the editor context, prepare a local copy
///     in editor and show form.
/// 3.) Else do not show form.
/// EditAction::New:
/// 1.) If an object id is given in query, navigate to edit with the given id.
/// 2.) Else if an existing object is loaded in the editor context OR no local object ID exists,
///     create a new local object in the editor context and show form.
/// 3.) Else do not show form.
pub fn set_up_editor_form<Q: ParamQueryId, EC: EditorContext>(editor_context: EC) -> Signal<bool> {
    let UseQueryNavigationReturn {
        url_matched_route_update_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let query_id = Q::use_param_query();
    let edit_action = EditActionParams::use_param_query();

    let (show_form, set_show_form) = signal(false);

    Effect::new({
        let navigate = navigate.clone();
        move || {
            match edit_action.get() {
                Some(EditAction::Edit) => {
                    // show form, if an address is loaded
                    set_show_form.set(editor_context.has_origin().get() && query_id.get().is_some());
                }
                Some(EditAction::Copy) => {
                    if let Some(id) = query_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            Q::KEY,
                            id.to_string().as_str(),
                            MatchedRouteHandler::ReplaceSegment(
                                EditAction::Edit.to_string().as_str(),
                            ),
                        );
                        navigate(
                            &nav_url,
                            NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    } else if editor_context.has_origin().get() {
                        // prepare copy in editor
                        editor_context.prepare_copy();
                        set_show_form.set(true);
                    } else if editor_context.has_id().get() && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if editor_context.has_id().get() {
                        // No origin, id is present, form is not shown -> show form
                        set_show_form.set(true);
                    } else {
                        // if there is no id, it means that no object was loaded, so we show the message to select an object from the list.
                        set_show_form.set(false);
                    }
                }
                Some(EditAction::New) => {
                    if let Some(id) = query_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            Q::KEY,
                            id.to_string().as_str(),
                            MatchedRouteHandler::ReplaceSegment(
                                EditAction::Edit.to_string().as_str(),
                            ),
                        );
                        navigate(
                            &nav_url,
                            NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    } else if editor_context.has_origin().get() || !editor_context.has_id().get() {
                        // if there is an origin or no id is set, create new object in editor and show form
                        editor_context.new_object();
                        set_show_form.set(true);
                    } else if editor_context.has_id().get() && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if editor_context.has_id().get() {
                        // No origin, id is present, form is not shown -> show form
                        set_show_form.set(true);
                    } else {
                        // if there is no id, it means that no object was loaded, so we show the message to select an object from the list.
                        set_show_form.set(false);
                    }
                }
                None => set_show_form.set(false),
            }
        }
    });

    show_form.into()
}
