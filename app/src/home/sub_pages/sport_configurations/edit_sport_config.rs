//! Sport Config Edit Module

use app_core::SportConfig;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::sport_config::{SaveSportConfigFormData, save_sport_config_inner};
use app_utils::{
    components::inputs::TextInputWithValidation,
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error, handle_write_error},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::{FilterNameQuery, ParamQuery, SportConfigIdQuery, SportIdQuery},
    server_fn::sport_config::{SaveSportConfig, load_sport_config},
    state::{
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        object_table_list::ObjectListContext,
        sport_config::SportConfigEditorContext,
        toast_state::ToastContext,
    },
};
use leptos::{html::H2, prelude::*};
#[cfg(feature = "test-mock")]
use leptos::{wasm_bindgen::JsCast, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_matched, use_navigate},
};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn LoadSportConfiguration() -> impl IntoView {
    // --- global state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- Server Resources ---
    let sport_config_id = SportConfigIdQuery::use_param_query();
    let sc_res = Resource::new(
        move || sport_config_id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match activity_tracker
                    .track_activity_wrapper(component_id.get_value(), load_sport_config(id))
                    .await
                {
                    Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                    load_result => load_result,
                },
                None => Ok(None),
            }
        },
    );

    let refetch = Callback::new(move |()| {
        sc_res.refetch();
    });

    let on_cancel = use_on_cancel();

    view! {
        <Transition fallback=move || {
            view! {
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex justify-center items-center p-4">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    </div>
                </div>
            }
        }>
            <ErrorBoundary fallback=move |errors| {
                for (_err_id, err) in errors.get().into_iter() {
                    let e = err.into_inner();
                    if let Some(app_err) = e.downcast_ref::<AppError>() {
                        handle_read_error(
                            &page_err_ctx,
                            component_id.get_value(),
                            app_err,
                            refetch,
                            on_cancel,
                        );
                    } else {
                        handle_general_error(
                            &page_err_ctx,
                            component_id.get_value(),
                            "An unexpected error occurred.",
                            None,
                            on_cancel,
                        );
                    }
                }
            }>
                // check if we have new or existing sport config
                {move || {
                    sc_res
                        .and_then(|may_be_sc| {
                            view! {
                                <EditSportConfiguration
                                    sport_config=may_be_sc.clone()
                                    refetch=refetch.clone()
                                />
                            }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn EditSportConfiguration(
    sport_config: Option<SportConfig>,
    refetch: Callback<()>,
) -> impl IntoView {
    // --- Hooks, Navigation & local and global state ---
    let UseQueryNavigationReturn {
        get_query,
        url_matched_route_update_query,
        url_matched_route_update_queries,
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let matched_route = use_matched();

    let sport_id = SportIdQuery::use_param_query();

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    let sport_config_list_ctx =
        expect_context::<ObjectListContext<SportConfig, SportConfigIdQuery>>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

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

    let sport_config_editor = SportConfigEditorContext::new();
    let (show_form, is_new) = if let Some(sc) = sport_config {
        sport_config_editor.set_sport_config(sc);
        (true, false)
    } else if let Some(s_id) = sport_id.get_untracked()
        && let Some(plugin) = sport_plugin_manager.get_untracked().get_web_ui(&s_id)
    {
        let mut sc = SportConfig::default();
        sc.set_sport_id(s_id);
        sc.set_config(plugin.get_default_config());
        sport_config_editor.set_sport_config(sc);
        let is_new = matched_route.get_untracked().ends_with("new");
        (is_new, is_new)
    } else {
        (false, false)
    };
    provide_context(sport_config_editor);

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // --- Server Actions ---
    let save_sport_config = ServerAction::<SaveSportConfig>::new();
    let save_sport_config_pending = save_sport_config.pending();
    activity_tracker.track_pending_memo(component_id.get_value(), save_sport_config_pending);

    // handle save result
    Effect::new(move || match save_sport_config.value().get() {
        Some(Ok(sc)) => {
            let sc_id = sc.get_id();
            save_sport_config.clear();
            toast_ctx.success("Sport Configuration saved successfully");
            if sport_config_list_ctx.is_id_in_list(sc_id) {
                let nav_url = url_matched_route_update_query(
                    SportConfigIdQuery::key(),
                    &sc_id.to_string(),
                    MatchedRouteHandler::RemoveSegment(1),
                );
                navigate(&nav_url, NavigateOptions::default());
                sport_config_list_ctx.trigger_refetch();
            } else {
                let refetch = get_query(FilterNameQuery::key()) != Some(sc.get_name().to_string());
                let sc_id = sc.get_id().to_string();
                let key_value = vec![
                    (SportConfigIdQuery::key(), sc_id.as_str()),
                    (FilterNameQuery::key(), sc.get_name()),
                ];
                let nav_url = url_matched_route_update_queries(
                    key_value,
                    MatchedRouteHandler::RemoveSegment(1),
                );
                navigate(&nav_url, NavigateOptions::default());
                if refetch {
                    sport_config_list_ctx.trigger_refetch();
                }
            }
        }
        Some(Err(err)) => {
            save_sport_config.clear();
            handle_write_error(
                &page_err_ctx,
                &toast_ctx,
                component_id.get_value(),
                &err,
                refetch,
            );
        }
        None => { /* saving state - do nothing */ }
    });

    // --- Signals for UI state & errors ---
    let is_disabled = move || sport_plugin().is_none() || save_sport_config_pending.get();

    let is_valid_config = move || {
        sport_config_editor.is_valid_json.get()
            && sport_config_editor.validation_result.with(|vr| vr.is_ok())
    };

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    {move || {
                        format!(
                            "{} {} Configuration",
                            if is_new { "New" } else { "Edit" },
                            sport_name(),
                        )
                    }}
                </h2>
                <Show
                    when=move || show_form
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
                                #[cfg(feature = "test-mock")]
                                {
                                    ev.prevent_default();
                                    let intent = ev
                                        .submitter()
                                        .and_then(|el| {
                                            el.dyn_into::<web_sys::HtmlButtonElement>().ok()
                                        })
                                        .map(|btn| btn.value());
                                    let data = SaveSportConfig {
                                        form: SaveSportConfigFormData {
                                            id: sport_config_editor
                                                .sport_config_id
                                                .get()
                                                .unwrap_or(Uuid::nil()),
                                            version: sport_config_editor
                                                .local_readonly
                                                .get()
                                                .map_or(0, |sc| sc.get_version().unwrap_or_default()),
                                            sport_id: sport_id.get().unwrap_or(Uuid::nil()),
                                            name: sport_config_editor.name.get().unwrap_or_default(),
                                            config: sport_config_editor
                                                .config
                                                .get()
                                                .unwrap_or_default()
                                                .to_string(),
                                            intent,
                                        },
                                    };
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
                                    save_action.dispatch(data);
                                }
                                #[cfg(not(feature = "test-mock"))]
                                {
                                    let _ = ev;
                                }
                            }
                        >
                            // --- Sport Config Form Fields ---
                            <fieldset class="space-y-4 contents" prop:disabled=is_disabled>
                                // Hidden meta fields the server expects (id / version)
                                <input
                                    type="hidden"
                                    name="form[id]"
                                    data-testid="hidden-id"
                                    prop:value=move || {
                                        sport_config_editor
                                            .sport_config_id
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
                                        sport_config_editor
                                            .local_readonly
                                            .get()
                                            .map_or(0, |sc| sc.get_version().unwrap_or_default())
                                    }
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
                                <TextInputWithValidation
                                    label="Name"
                                    name="form[name]"
                                    data_testid="input-name"
                                    value=sport_config_editor.name
                                    set_value=sport_config_editor.set_name
                                    validation_result=sport_config_editor.validation_result
                                    object_id=sport_config_editor.sport_config_id
                                    field="name"
                                />
                                // Sport specific configuration UI
                                {move || {
                                    sport_plugin().map(|plugin| plugin.render_configuration())
                                }}
                                // buttons
                                <div class="card-actions justify-end mt-4">
                                    <button
                                        type="submit"
                                        name="form[intent]"
                                        value=move || if is_new { "create" } else { "update" }
                                        data-testid="btn-save"
                                        class="btn btn-primary"
                                        prop:disabled=move || {
                                            is_disabled() || !is_valid_config()
                                        }
                                    >
                                        "Save"
                                    </button>

                                    <button
                                        type="submit"
                                        name="form[intent]"
                                        value="copy_as_new"
                                        data-testid="btn-save-as-new"
                                        class="btn btn-secondary"
                                        prop:disabled=move || {
                                            is_disabled() || is_new || !is_valid_config()
                                        }
                                        prop:hidden=move || is_new
                                    >
                                        "Save as new"
                                    </button>

                                    <button
                                        type="button"
                                        name="form[intent]"
                                        value="cancel"
                                        data-testid="btn-cancel"
                                        class="btn btn-ghost"
                                        on:click=move |_| on_cancel.run(())
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            </fieldset>
                        </ActionForm>
                    </div>
                </Show>
            </div>
        </div>
    }
}
