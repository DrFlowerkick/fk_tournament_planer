//! Sport Config Edit Module

use app_core::{SportConfig, utils::validation::FieldResult};
use app_utils::{
    components::inputs::ValidatedTextInput,
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error, handle_write_error},
    },
    hooks::{
        is_field_valid::is_field_valid,
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportConfigParams, SportParams},
    server_fn::sport_config::{SaveSportConfig, load_sport_config},
    state::{
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        toast_state::{ToastContext, ToastVariant},
    },
};
use leptos::{logging::log, prelude::*};
#[cfg(feature = "test-mock")]
use leptos::{wasm_bindgen::JsCast, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use reactive_stores::Store;
use serde_json::Value;
use shared::{RenderCfgProps, SportPortWebUi};
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn SportConfigForm() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_with_path,
        url_with_update_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let sport_query = use_query::<SportParams>();
    let sport_id = Signal::derive(move || sport_query.get().map(|s| s.sport_id).unwrap_or(None));

    let sport_config_query = use_query::<SportConfigParams>();
    let sport_config_id = Signal::derive(move || {
        sport_config_query
            .get()
            .ok()
            .and_then(|sc| sc.sport_config_id)
    });

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    let state = expect_context::<Store<GlobalState>>();
    let return_after_sport_config_edit = state.return_after_sport_config_edit();
    let sport_plugin_manager = state.sport_plugin_manager();

    let cancel_target =
        Callback::new(move |_: ()| url_with_path(&return_after_sport_config_edit.get()));

    let sport_plugin = move || {
        if let Ok(sport_params) = sport_query.get()
            && let Some(sport_id) = sport_params.sport_id
        {
            sport_plugin_manager.get().get_web_ui(&sport_id)
        } else {
            log!("No valid sport_id in query params. Editing sport config is disabled.");
            None
        }
    };

    let sport_name = move || {
        if let Some(plugin) = sport_plugin() {
            plugin.name()
        } else {
            "Unknown Sport"
        }
    };

    // --- Signals for form fields ---
    let (is_new, set_is_new) = signal(false);
    let set_name = RwSignal::new(String::new());
    let set_sport_config = RwSignal::new(None::<Value>);
    let (id, set_id) = signal(Uuid::nil());
    let set_version = RwSignal::new(0);

    // --- Server Actions & Resources ---
    let save_sport_config = ServerAction::<SaveSportConfig>::new();

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

    let refetch_and_reset = Callback::new(move |()| {
        save_sport_config.clear();
        sc_res.refetch();
    });

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // handle save result
    Effect::new(move || match save_sport_config.value().get() {
        Some(Ok(sc)) => {
            save_sport_config.clear();
            toast_ctx.add(
                "Sport Configuration saved successfully",
                ToastVariant::Success,
            );
            let nav_url = url_with_update_query(
                "sport_config_id",
                &sc.get_id().to_string(),
                Some(&return_after_sport_config_edit.get()),
            );
            navigate(&nav_url, NavigateOptions::default());
        }
        Some(Err(err)) => {
            handle_write_error(
                &page_err_ctx,
                &toast_ctx,
                component_id.get_value(),
                &err,
                refetch_and_reset,
            );
        }
        None => { /* saving state - do nothing */ }
    });

    // --- Signals for UI state & errors ---
    // reset these signals with save_sport_config.clear() when needed
    let pending = save_sport_config.pending();

    // ToDo: refactor error handling to avoid duplication with above Effect
    let is_conflict = move || {
        if let Some(Err(AppError::Core(ce))) = save_sport_config.value().get()
            && ce.is_optimistic_lock_conflict()
        {
            true
        } else {
            false
        }
    };
    let is_duplicate = move || {
        if let Some(Err(AppError::Core(ce))) = save_sport_config.value().get()
            && ce.is_unique_violation()
        {
            true
        } else {
            false
        }
    };
    let is_addr_res_error = move || matches!(sc_res.get(), Some(Err(_)));
    let is_general_error = move || {
        if let Some(Err(err)) = save_sport_config.value().get() {
            match err {
                AppError::Core(ref ce) => {
                    if ce.is_optimistic_lock_conflict() || ce.is_unique_violation() {
                        None
                    } else {
                        Some(format!("{:?}", ce))
                    }
                }
                _ => Some(format!("{:?}", err)),
            }
        } else {
            None
        }
    };

    let is_disabled = move || {
        sport_plugin().is_none()
            || sc_res.get().is_none()
            || pending.get()
            || is_conflict()
            || is_duplicate()
            || is_addr_res_error()
            || is_general_error().is_some()
    };

    // --- Validation & Normalization ---
    let current_config = Memo::new(move |_| {
        if let Some(s_id) = sport_id.get()
            && let Some(cfg) = set_sport_config.get()
        {
            let mut config = SportConfig::default();
            config
                .set_name(set_name.get())
                .set_sport_id(s_id)
                .set_config(cfg);
            Some(config)
        } else {
            None
        }
    });

    // --- validation signals ---
    let is_valid_json = RwSignal::new(false);

    // This only validates the non json fields, which is currently only "name"
    let validation_result = Signal::derive(move || {
        if let Some(cfg) = current_config.get() {
            cfg.validate()
        } else {
            Ok(())
        }
    });

    let is_valid_config =
        Signal::derive(move || validation_result.get().is_ok() && is_valid_json.get());

    let is_valid_name = Signal::derive(move || is_field_valid(validation_result, "name"));

    let props = FormFieldsProperties {
        id: id.into(),
        sport_id,
        sport_plugin: Signal::derive(sport_plugin),
        cancel_target,
        is_disabled: Signal::derive(is_disabled),
        is_new: is_new.into(),
        set_name,
        set_sport_config,
        set_version,
        is_valid_name,
        is_valid_json,
        is_valid_config,
        current_config,
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">
                    {move || {
                        format!(
                            "{} {} Configuration",
                            if is_new.get() { "New" } else { "Edit" },
                            sport_name(),
                        )
                    }}
                </h2>
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
                                    refetch_and_reset,
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
                                    match may_be_sc {
                                        Some(sport_config) => {
                                            set_name.set(sport_config.get_name().to_string());
                                            set_sport_config
                                                .set(Some(sport_config.get_config().clone()));
                                            set_id.set(sport_config.get_id());
                                            set_version
                                                .set(sport_config.get_version().unwrap_or_default());
                                            set_is_new.set(false);
                                        }
                                        None => {
                                            if let Some(plugin) = sport_plugin() {
                                                let plugin_config = plugin.get_default_config();
                                                set_name.set("".into());
                                                set_sport_config.set(Some(plugin_config));
                                                set_id.set(Uuid::new_v4());
                                                set_version.set(0);
                                                set_is_new.set(true);
                                            }
                                        }
                                    };
                                    view! {
                                        // --- Sport Config Form ---
                                        <div data-testid="form-sport-config">
                                            {
                                                #[cfg(not(feature = "test-mock"))]
                                                {
                                                    view! {
                                                        <ActionForm action=save_sport_config>
                                                            <FormFields props=props />
                                                        </ActionForm>
                                                    }
                                                }
                                                #[cfg(feature = "test-mock")]
                                                {
                                                    view! {
                                                        <form on:submit=move |ev| {
                                                            ev.prevent_default();
                                                            let intent = ev
                                                                .submitter()
                                                                .and_then(|el| {
                                                                    el.dyn_into::<web_sys::HtmlButtonElement>().ok()
                                                                })
                                                                .map(|btn| btn.value());
                                                            let data = SaveSportConfig {
                                                                id: sport_config_id.get().unwrap_or(Uuid::nil()),
                                                                version: set_version.get(),
                                                                sport_id: sport_id.get().unwrap_or(Uuid::nil()),
                                                                name: set_name.get(),
                                                                config: set_sport_config
                                                                    .get()
                                                                    .unwrap_or_default()
                                                                    .to_string(),
                                                                intent,
                                                            };
                                                            save_sport_config.dispatch(data);
                                                        }>
                                                            <FormFields props=props />
                                                        </form>
                                                    }
                                                }
                                            }
                                        </div>
                                    }
                                })
                        }}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
    }
}

// Props for form fields component
#[derive(Clone, Copy)]
struct FormFieldsProperties {
    id: Signal<Uuid>,
    sport_id: Signal<Option<Uuid>>,
    sport_plugin: Signal<Option<Arc<dyn SportPortWebUi>>>,
    cancel_target: Callback<(), String>,
    is_disabled: Signal<bool>,
    is_new: Signal<bool>,
    set_name: RwSignal<String>,
    set_sport_config: RwSignal<Option<Value>>,
    set_version: RwSignal<u32>,
    is_valid_name: Signal<FieldResult<()>>,
    is_valid_json: RwSignal<bool>,
    is_valid_config: Signal<bool>,
    current_config: Memo<Option<SportConfig>>,
}

#[component]
fn FormFields(props: FormFieldsProperties) -> impl IntoView {
    let FormFieldsProperties {
        id,
        sport_id,
        sport_plugin,
        cancel_target,
        is_disabled,
        is_new,
        set_name,
        set_sport_config,
        set_version,
        is_valid_name,
        is_valid_json,
        is_valid_config,
        current_config,
    } = props;
    let navigate = use_navigate();

    let props = RenderCfgProps {
        object_id: id,
        config: set_sport_config,
        is_valid_json,
    };

    view! {
        // --- Sport Config Form Fields ---
        <fieldset class="space-y-4" prop:disabled=is_disabled>
            // Hidden meta fields the server expects (id / version)
            <input
                type="hidden"
                name="id"
                data-testid="hidden-id"
                prop:value=move || id.get().to_string()
            />
            <input
                type="hidden"
                name="version"
                data-testid="hidden-version"
                prop:value=set_version
            />
            <input
                type="hidden"
                name="sport_id"
                data-testid="hidden-sport-id"
                prop:value=move || sport_id.get().unwrap_or_default().to_string()
            />
            <input
                type="hidden"
                name="config"
                data-testid="hidden-sport-config"
                prop:value=move || set_sport_config.get().unwrap_or_default().to_string()
            />
            <ValidatedTextInput
                label="Name"
                name="name"
                value=set_name
                validation_error=is_valid_name
                is_new=is_new
                on_blur=move || {
                    if let Some(cfg) = current_config.get() {
                        set_name.set(cfg.get_name().to_string());
                    }
                }
            />
            // Sport specific configuration UI
            {move || { sport_plugin.get().map(|plugin| plugin.render_configuration(props)) }}
            // buttons
            <div class="card-actions justify-end mt-4">
                <button
                    type="submit"
                    name="intent"
                    value=move || if is_new.get() { "create" } else { "update" }
                    data-testid="btn-save"
                    class="btn btn-primary"
                    prop:disabled=move || is_disabled.get() || !is_valid_config.get()
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
                        is_disabled.get() || is_new.get() || !is_valid_config.get()
                    }
                    prop:hidden=move || is_new.get()
                >
                    "Save as new"
                </button>

                <button
                    type="button"
                    name="intent"
                    value="cancel"
                    data-testid="btn-cancel"
                    class="btn btn-ghost"
                    on:click=move |_| navigate(&cancel_target.run(()), NavigateOptions::default())
                >
                    "Cancel"
                </button>
            </div>
        </fieldset>
    }
}
