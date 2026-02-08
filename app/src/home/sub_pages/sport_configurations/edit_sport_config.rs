//! Sport Config Edit Module

use app_core::SportConfig;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::sport_config::save_sport_config_inner;
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
    params::{use_sport_config_id_query, use_sport_id_query},
    server_fn::sport_config::{SaveSportConfig, load_sport_config},
    state::{
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        sport_config::{SportConfigEditorContext, SportConfigListContext},
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
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // --- Server Resources ---
    let sport_config_id = use_sport_config_id_query();
    let sc_res = Resource::new(
        move || sport_config_id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_sport_config(id).await {
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
                <div class="flex justify-center items-center p-4">
                    <span class="loading loading-spinner loading-lg"></span>
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
        url_matched_route_update_query,
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let matched_route = use_matched();

    let sport_id = use_sport_id_query();

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    let sport_config_list_ctx = expect_context::<SportConfigListContext>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
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

    // handle save result
    Effect::new(move || match save_sport_config.value().get() {
        Some(Ok(sc)) => {
            save_sport_config.clear();
            toast_ctx.success("Sport Configuration saved successfully");
            if is_new {
                sport_config_list_ctx.trigger_refetch();
            }
            let nav_url = url_matched_route_update_query(
                "sport_config_id",
                &sc.get_id().to_string(),
                MatchedRouteHandler::RemoveSegment(1),
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

    let save_sport_config_pending = save_sport_config.pending();

    // --- Signals for UI state & errors ---
    // use try, because these signals are use in conjunction with page_err_ctx,
    // which has another "lifetime" in the reactive system, which may cause panics
    // for the other signals when the component is unmounted.
    let is_disabled = move || {
        sport_plugin().is_none()
            || save_sport_config_pending.try_get().unwrap_or(false)
            || page_err_ctx.has_errors()
    };

    let is_valid_config = move || {
        sport_config_editor.is_valid_json.try_get().unwrap_or(false)
            && sport_config_editor
                .validation_result
                .try_with(|vr| vr.is_ok())
                .unwrap_or(false)
    };

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                // ToDo: header as part of card?
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
                                    };
                                    let save_action = Action::new(|sc: &SaveSportConfig| {
                                        let sc = sc.clone();
                                        async move {
                                            save_sport_config_inner(
                                                    sc.id,
                                                    sc.version,
                                                    sc.sport_id,
                                                    sc.name,
                                                    sc.config,
                                                    sc.intent,
                                                )
                                                .await
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
                            <fieldset class="space-y-4" prop:disabled=is_disabled>
                                // Hidden meta fields the server expects (id / version)
                                <input
                                    type="hidden"
                                    name="id"
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
                                    name="version"
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
                                    name="sport_id"
                                    data-testid="hidden-sport-id"
                                    prop:value=move || {
                                        sport_id.get().unwrap_or_default().to_string()
                                    }
                                />
                                <input
                                    type="hidden"
                                    name="config"
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
                                    name="name"
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
                                        name="intent"
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
                                        name="intent"
                                        value="create"
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
                                        name="intent"
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
