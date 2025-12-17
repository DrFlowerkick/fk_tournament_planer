//! Sport Config Edit Module

use app_core::SportConfig;
use app_utils::{
    components::{
        banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner},
        inputs::ValidatedTextInput,
    },
    error::AppError,
    global_state::{GlobalState, GlobalStateStoreFields},
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::{SportConfigParams, SportParams},
    server_fn::sport_config::{SaveSportConfig, load_sport_config},
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
use shared::{RenderCfgProps, SportConfigWebUi};
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn SportConfigForm() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        update,
        path,
        query_string,
        ..
    } = use_query_navigation();

    let sport_query = use_query::<SportParams>();
    let sport_id = Signal::derive(move || sport_query.get().map(|s| s.sport_id).unwrap_or(None));

    let is_new = move || path.read().ends_with("/new_sc") || path.read().is_empty();
    let sport_config_query = use_query::<SportConfigParams>();
    let sport_config_id = Signal::derive(move || {
        if is_new() {
            None
        } else {
            sport_config_query
                .get()
                .map(|sc| sc.sport_config_id)
                .unwrap_or(None)
        }
    });

    let state = expect_context::<Store<GlobalState>>();
    let return_after_sport_config_edit = state.return_after_sport_config_edit();
    let sport_plugin_manager = state.sport_plugin_manager();

    let cancel_target = Callback::new(move |_: ()| {
        format!(
            "{}{}",
            return_after_sport_config_edit.get(),
            query_string.get()
        )
    });

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
    let set_name = RwSignal::new(String::new());
    let set_sport_config = RwSignal::new(None::<Value>);
    let set_version = RwSignal::new(0);

    // --- Server Actions & Resources ---
    let save_sport_config = ServerAction::<SaveSportConfig>::new();

    Effect::new(move || {
        if let Some(Ok(sc)) = save_sport_config.value().get() {
            save_sport_config.clear();
            update(
                "sport_config_id",
                &sc.get_id().map(|id| id.to_string()).unwrap_or_default(),
            );
            let nav_url = format!(
                "{}{}",
                return_after_sport_config_edit.get(),
                query_string.get()
            );
            let navigate = use_navigate();
            navigate(&nav_url, NavigateOptions::default());
        }
    });

    let sc_res = Resource::new(
        move || sport_config_id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_sport_config(id).await {
                    Ok(Some(sc)) => {
                        set_name.set(sc.get_name().to_string());
                        set_sport_config.set(Some(sc.get_config().clone()));
                        set_version.set(sc.get_version().unwrap_or_default());
                        Ok(sc)
                    }
                    Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                    Err(e) => Err(e),
                },
                None => Ok(Default::default()),
            }
        },
    );

    let refetch_and_reset = move || {
        save_sport_config.clear();
        sc_res.refetch();
    };

    // --- Signals for UI state & errors ---
    // reset these signals with save_sport_config.clear() when needed
    let pending = save_sport_config.pending();

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

    let props = FormFieldsProperties {
        id: sport_config_id,
        sport_id,
        sport_plugin: Signal::derive(sport_plugin),
        sc_res,
        cancel_target,
        is_disabled: Signal::derive(is_disabled),
        is_new: Signal::derive(is_new),
        set_name,
        set_sport_config,
        set_version,
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">
                    {move || { if is_new() { "New Sport Config" } else { "Edit Sport Config" } }}
                </h2>
                <Transition fallback=move || {
                    view! {
                        <div class="flex justify-center items-center p-4">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    }
                }>
                    {move || {
                        sc_res
                            .get()
                            .map(|res| match res {
                                Err(msg) => {
                                    // --- General Load Error Banner ---
                                    view! {
                                        <AcknowledgmentAndNavigateBanner
                                            msg=format!(
                                                "An unexpected error occurred during load: {msg}",
                                            )
                                            ack_btn_text="Reload"
                                            ack_action=refetch_and_reset
                                            nav_btn_text="Cancel"
                                            navigate_url=cancel_target.run(())
                                        />
                                    }
                                        .into_any()
                                }
                                Ok(_addr) => {
                                    view! {
                                        // --- Conflict Banner ---
                                        {move || {
                                            if is_conflict() {
                                                view! {
                                                    <AcknowledgmentBanner
                                                        msg="A newer version of this sport configuration exists. Reloading will discard your changes."
                                                        ack_btn_text="Reload"
                                                        ack_action=refetch_and_reset.clone()
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}

                                        // --- Duplicate Banner ---
                                        {move || {
                                            if is_duplicate() {
                                                view! {
                                                    <AcknowledgmentBanner
                                                        msg=format!(
                                                            "A sport configuration with name '{}' already exists for '{}'. ",
                                                            set_name.get(),
                                                            sport_name(),
                                                        )
                                                        ack_btn_text="Ok"
                                                        ack_action=move || save_sport_config.clear()
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                        // --- General Save Error Banner ---
                                        {move || {
                                            if let Some(msg) = is_general_error() {
                                                view! {
                                                    <AcknowledgmentAndNavigateBanner
                                                        msg=format!(
                                                            "An unexpected error occurred during saving: {msg}",
                                                        )
                                                        ack_btn_text="Dismiss"
                                                        ack_action=move || save_sport_config.clear()
                                                        nav_btn_text="Return to Search Sport Config"
                                                        navigate_url=cancel_target.run(())
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
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
                                                                config: set_sport_config.get().unwrap_or_default(),
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
                                        .into_any()
                                }
                            })
                    }}

                </Transition>
            </div>
        </div>
    }
}

// Props for form fields component
#[derive(Clone, Copy)]
struct FormFieldsProperties {
    id: Signal<Option<Uuid>>,
    sport_id: Signal<Option<Uuid>>,
    sport_plugin: Signal<Option<Arc<dyn SportConfigWebUi>>>,
    sc_res: Resource<Result<SportConfig, AppError>>,
    cancel_target: Callback<(), String>,
    is_disabled: Signal<bool>,
    is_new: Signal<bool>,
    set_name: RwSignal<String>,
    set_sport_config: RwSignal<Option<Value>>,
    set_version: RwSignal<u32>,
}

#[component]
fn FormFields(props: FormFieldsProperties) -> impl IntoView {
    let FormFieldsProperties {
        id,
        sport_id,
        sport_plugin,
        sc_res,
        cancel_target,
        is_disabled,
        is_new,
        set_name,
        set_sport_config,
        set_version,
    } = props;
    let navigate = use_navigate();

    // --- Derived Signal for Validation & Normalization ---
    let current_config = move || {
        let mut config = SportConfig::default();
        config
            .set_name(set_name.get())
            .set_sport_id(sport_id.get().unwrap_or_default())
            .set_config(set_sport_config.get().unwrap_or_default());
        config
    };

    let is_loading = Signal::derive(move || sc_res.get().is_none());

    // --- validation signals ---
    let is_valid_json = RwSignal::new(false);

    // This only validates the non json fields, which is currently only "name"
    let validation_result = move || current_config().validate();
    let is_valid_config = move || validation_result().is_ok() && is_valid_json.get();

    // --- Simplified Validation Closures ---
    let is_field_valid = move |field: &str| match validation_result() {
        Ok(_) => true,
        Err(err) => err.errors.iter().all(|e| e.get_field() != field),
    };

    let is_valid_name = Signal::derive(move || is_field_valid("name"));

    let props = RenderCfgProps {
        config: set_sport_config,
        is_valid_json,
        is_new,
        is_loading,
    };

    view! {
        // --- Sport Config Form Fields ---
        <fieldset class="space-y-4" prop:disabled=is_disabled>
            // Hidden meta fields the server expects (id / version)
            <input
                type="hidden"
                name="id"
                data-testid="hidden-id"
                prop:value=move || id.get().unwrap_or(Uuid::nil()).to_string()
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
            <ValidatedTextInput
                label="Name"
                name="name"
                value=set_name
                is_valid=is_valid_name
                is_loading=is_loading
                is_new=is_new
                on_blur=move || set_name.set(current_config().get_name().to_string())
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
                    prop:disabled=move || is_disabled.get() || !is_valid_config()
                >
                    "Save"
                </button>

                <button
                    type="submit"
                    name="intent"
                    value="create"
                    data-testid="btn-save-as-new"
                    class="btn btn-secondary"
                    prop:disabled=move || is_disabled.get() || is_new.get() || !is_valid_config()
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
