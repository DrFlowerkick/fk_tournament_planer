//! Sport Config Edit Module

#[cfg(feature = "test-mock")]
use app_utils::server_fn::sport_config::save_sport_config_inner;
use app_utils::{
    components::inputs::{InputCommitAction, TextInput},
    enum_utils::EditAction,
    error::strategy::handle_write_error,
    hooks::{
        //set_up_editor_form::set_up_editor_form,
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::{EditActionParams, FilterNameQuery, ParamQuery, SportConfigIdQuery, SportIdQuery},
    server_fn::sport_config::{SaveSportConfig, SaveSportConfigFormData},
    state::{
        EditorContext,
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        sport_config::SportConfigEditorContext,
        toast_state::ToastContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn EditSportConfiguration() -> impl IntoView {
    // --- Hooks, Navigation & local and global state ---
    let UseQueryNavigationReturn {
        url_matched_route_update_query,
        url_matched_route_update_queries,
        url_is_matched_route,
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

    // sport id and plugin manager
    let sport_id = SportIdQuery::use_param_query();
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .try_with(|maybe_sport_id| {
                maybe_sport_id
                    .as_ref()
                    .and_then(|s_id| sport_plugin_manager.get().get_web_ui(s_id))
            })
            .flatten()
    };
    let sport_name = move || {
        if let Some(plugin) = sport_plugin() {
            plugin.name()
        } else {
            "Unknown Sport"
        }
    };
    let sport_config_id = SportConfigIdQuery::use_param_query();

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- local state ---
    let sport_config_editor = expect_context::<SportConfigEditorContext>();

    // --- state initialization & effects ---
    //let show_form =
    //    set_up_editor_form::<SportConfigIdQuery, SportConfigEditorContext>(sport_config_editor);
    let (show_form, set_show_form) = signal(false);

    Effect::new({
        let navigate = navigate.clone();
        move || {
            match edit_action.get() {
                Some(EditAction::Edit) => {
                    // show form, if an address is loaded
                    set_show_form.set(sport_config_editor.has_origin().get() && sport_config_id.get().is_some());
                }
                Some(EditAction::Copy) => {
                    if let Some(id) = sport_config_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            SportConfigIdQuery::KEY,
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
                    } else if sport_config_editor.has_origin().get() {
                        // prepare copy in editor
                        sport_config_editor.prepare_copy();
                        set_show_form.set(true);
                    } else if sport_config_editor.has_id().get() && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if sport_config_editor.has_id().get() {
                        // No origin, id is present, form is not shown -> show form
                        set_show_form.set(true);
                    } else {
                        // if there is no id, it means that no object was loaded, so we show the message to select an object from the list.
                        set_show_form.set(false);
                    }
                }
                Some(EditAction::New) => {
                    if let Some(id) = sport_config_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            SportConfigIdQuery::KEY,
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
                    } else if sport_config_editor.has_origin().get() || !sport_config_editor.has_id().get() {
                        // if there is an origin or no id is set, create new object in editor and show form
                        sport_config_editor.new_object();
                        set_show_form.set(true);
                    } else if sport_config_editor.has_id().get() && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if sport_config_editor.has_id().get() {
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

    let origin = move || {
        if sport_config_editor.has_origin().get() {
            "Has Origin".to_string()
        } else {
            "No Origin".to_string()
        }
    };

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // --- Server Actions ---
    let save_sport_config = ServerAction::<SaveSportConfig>::new();
    let save_sport_config_pending = save_sport_config.pending();
    activity_tracker.track_pending_memo(component_id.get_value(), save_sport_config_pending);

    // ToDo: with auto save and parallel editing, refetch is done automatically. Delete this dummy refetch.
    let refetch = Callback::new(move |_| {});

    // handle save result
    Effect::new(move || {
        if let Some(ssc_result) = save_sport_config.value().get()
            && let Some(edit_action) = edit_action.get()
        {
            save_sport_config.clear();

            match ssc_result {
                Ok(sc) => {
                    match edit_action {
                        EditAction::New | EditAction::Copy => {
                            let sc_id = sc.get_id().to_string();
                            let key_value = vec![
                                (SportConfigIdQuery::KEY, sc_id.as_str()),
                                (FilterNameQuery::KEY, sc.get_name()),
                            ];
                            let nav_url = url_matched_route_update_queries(
                                key_value,
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
                        }
                        EditAction::Edit => {
                            // ToDo: after some more testing we ca probably remove this
                            if !sport_config_editor.check_optimistic_version(sc.get_version()) {
                                // version mismatch, likely due to parallel editing
                                // this should not happen, because version mismatch should be caught
                                // by the server and returned as error, but we handle it here just in case
                                leptos::logging::log!(
                                    "Version mismatch after saving Sport Config. Expected version: {:?}, actual version: {:?}. This might be caused by parallel editing.",
                                    sport_config_editor.version.get(),
                                    sc.get_version()
                                );
                            }
                        }
                    }
                    sport_config_editor.set_sport_config(sc);
                }
                Err(err) => {
                    save_sport_config.clear();
                    handle_write_error(
                        &page_err_ctx,
                        &toast_ctx,
                        component_id.get_value(),
                        &err,
                        refetch,
                    );
                }
            }
        }
    });

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
                                Some(EditAction::New) => {
                                    "New Sport Configuration for ".to_string() + sport_name()
                                }
                                Some(EditAction::Edit) => {
                                    "Edit Sport Configuration for ".to_string() + sport_name()
                                }
                                Some(EditAction::Copy) => {
                                    "Copy Sport Configuration for ".to_string() + sport_name()
                                }
                                None => "".to_string(),
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
                    <p>{move || origin()}</p>
                    <Show
                        when=move || show_form.get()
                        fallback=|| {
                            view! {
                                <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                    <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                    <p class="text-2xl font-bold text-center">
                                        "Please select a sport configuration from the list."
                                    </p>
                                </div>
                            }
                        }
                    >
                        // --- Sport Config Form ---
                        // create on_submit fn in server fn file
                        // for this to work we need a SportConfigEditorContext for signal handling
                        <div data-testid="form-sport-config">
                            <ActionForm
                                action=save_sport_config
                                on:submit:capture=move |ev| {
                                    ev.prevent_default();
                                    if sport_config_editor.validation_result.with(|vr| vr.is_err())
                                    {
                                        return;
                                    }
                                    sport_config_editor.increment_version();
                                    let form_data = SaveSportConfig {
                                        form: SaveSportConfigFormData {
                                            id: sport_config_editor.id.get().unwrap_or(Uuid::nil()),
                                            version: sport_config_editor
                                                .version
                                                .get()
                                                .unwrap_or_default(),
                                            sport_id: sport_id.get().unwrap_or(Uuid::nil()),
                                            name: sport_config_editor.name.get().unwrap_or_default(),
                                            config: sport_config_editor
                                                .config
                                                .get()
                                                .unwrap_or_default()
                                                .to_string(),
                                            intent: intent.get(),
                                        },
                                    };
                                    #[cfg(feature = "test-mock")]
                                    {
                                        let save_action = Action::new(|sc: &SaveSportConfig| {
                                            let sc = sc.clone();
                                            async move {
                                                let result = save_sport_config_inner(sc.form).await;
                                                leptos::web_sys::console::log_1(
                                                    &format!("Result of save sport config: {:?}", result).into(),
                                                );
                                                result
                                            }
                                        });
                                        save_action.dispatch(form_data);
                                    }
                                    #[cfg(not(feature = "test-mock"))]
                                    {
                                        save_sport_config.dispatch(form_data);
                                    }
                                }
                            >
                                // --- Sport Config Form Fields ---
                                <fieldset class="space-y-4 contents">
                                    // Hidden meta fields the server expects (id / version)
                                    <input
                                        type="hidden"
                                        name="form[id]"
                                        data-testid="hidden-id"
                                        prop:value=move || {
                                            sport_config_editor
                                                .id
                                                .get()
                                                .unwrap_or(Uuid::nil())
                                                .to_string()
                                        }
                                    />
                                    <input
                                        type="hidden"
                                        name="form[version]"
                                        data-testid="hidden-version"
                                        prop:value=move || {
                                            sport_config_editor.version.get().unwrap_or_default()
                                        }
                                    />
                                    <input
                                        type="hidden"
                                        name="form[intent]"
                                        data-testid="intent"
                                        prop:value=move || intent.get()
                                    />
                                    <input
                                        type="hidden"
                                        name="form[sport_id]"
                                        data-testid="hidden-sport-id"
                                        prop:value=move || {
                                            sport_id.get().unwrap_or_default().to_string()
                                        }
                                    />
                                    <input
                                        type="hidden"
                                        name="form[config]"
                                        data-testid="hidden-sport-config"
                                        prop:value=move || {
                                            sport_config_editor
                                                .config
                                                .get()
                                                .unwrap_or_default()
                                                .to_string()
                                        }
                                    />
                                    <TextInput
                                        label="Name"
                                        name="form[name]"
                                        data_testid="input-name"
                                        value=sport_config_editor.name
                                        action=InputCommitAction::WriteAndSubmit(
                                            sport_config_editor.set_name,
                                        )
                                        validation_result=sport_config_editor.validation_result
                                        object_id=sport_config_editor.id
                                        field="name"
                                    />
                                    // Sport specific configuration UI
                                    {move || {
                                        sport_plugin().map(|plugin| plugin.render_configuration())
                                    }}
                                </fieldset>
                            </ActionForm>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
