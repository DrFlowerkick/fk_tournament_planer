//! sport configuration editor context

use crate::{
    error::{AppError, strategy::handle_with_toast},
    params::{ParamQuery, SportIdQuery},
    server_fn::sport_config::{LoadSportConfig, SaveSportConfig},
    state::{
        EditorContext, EditorContextWithResource, LabeledAction, SimpleEditorOptions,
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        toast_state::ToastContext,
    },
};
use app_core::{
    CrTopic, SportConfig,
    utils::{
        id_version::IdVersion,
        validation::{FieldError, ValidationResult},
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::prelude::*;
use reactive_stores::Store;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct SportConfigEditorContext {
    // --- state & derived signals ---
    /// The local editable sport configuration.
    local: RwSignal<Option<SportConfig>>,
    /// Read slice of local sport configuration for use in editor components
    pub local_read_only: Signal<Option<SportConfig>>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,
    /// WriteSignal for setting a unique violation error on the name field, if any
    pub set_unique_violation_error: WriteSignal<Option<FieldError>>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the id field
    pub id: Signal<Option<Uuid>>,
    /// Signal slice for the version field
    pub version: Signal<Option<u32>>,
    /// Signal slice for the name field
    pub name: Signal<Option<String>>,
    /// Callback for updating the name field
    pub set_name: Callback<Option<String>>,
    /// Signal slice for the config field
    pub config: Signal<Option<Value>>,
    /// SignalSetter for updating the config field
    pub set_config: SignalSetter<Value>,

    // --- Resource & server action state ---
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    set_optimistic_version: RwSignal<Option<u32>>,
    /// Server action for saving the sport configuration based on the current state of the editor context
    pub save_sport_config: ServerAction<SaveSportConfig>,
    /// Callback after successful save to e.g. navigate to the new sport configuration or show a success toast.
    pub post_save_callback: StoredValue<Option<Callback<SportConfig>>>,
}

impl EditorContext for SportConfigEditorContext {
    type ObjectType = SportConfig;
    type NewEditorOptions = SimpleEditorOptions;

    /// Create a new `SportConfigEditorContext`.
    fn new(options: SimpleEditorOptions) -> Self {
        // ---- global state & context ----
        let toast_ctx = expect_context::<ToastContext>();
        let page_err_ctx = expect_context::<PageErrorContext>();
        let activity_tracker = expect_context::<ActivityTracker>();
        let component_id = StoredValue::new(Uuid::new_v4());
        // remove errors on unmount
        on_cleanup(move || {
            page_err_ctx.clear_all_for_component(component_id.get_value());
            activity_tracker.remove_component(component_id.get_value());
        });

        // ---- signals & slices ----
        let local = RwSignal::new(None::<SportConfig>);

        let sport_id = SportIdQuery::use_param_query();
        let state = expect_context::<Store<GlobalState>>();
        let sport_plugin_manager = state.sport_plugin_manager();
        let sport_plugin = move || {
            sport_id
                .get()
                .and_then(|id| sport_plugin_manager.get().get_web_ui(&id))
        };
        let (unique_violation_error, set_unique_violation_error) = signal(None::<FieldError>);
        let validation_result = Signal::derive(move || {
            let vr = local.with(|local| {
                if let Some(sc) = local
                    && let Some(plugin) = sport_plugin()
                {
                    sc.validate(plugin)
                } else {
                    ValidationResult::Ok(())
                }
            });
            if let Some(unique_err) = unique_violation_error.get() {
                if let Err(mut validation_errors) = vr {
                    validation_errors.add(unique_err);
                    Err(validation_errors)
                } else {
                    Err(unique_err.into())
                }
            } else {
                vr
            }
        });

        let id = create_read_slice(local, move |local| local.as_ref().map(|sc| sc.get_id()));
        let version = create_read_slice(local, move |local| {
            local.as_ref().and_then(|sc| sc.get_version())
        });
        let (name, set_name) = create_slice(
            local,
            |local| local.as_ref().map(|sc| sc.get_name().to_string()),
            move |local, name: String| {
                if let Some(sc) = local {
                    sc.set_name(name);
                    // Clear unique violation error on name change, if any
                    set_unique_violation_error.set(None);
                }
            },
        );
        let set_name = Callback::new(move |name: Option<String>| {
            set_name.set(name.unwrap_or_default());
        });
        let (config, set_config) = create_slice(
            local,
            |local| local.as_ref().map(|sc| sc.get_config().clone()),
            |local, config: Value| {
                if let Some(sc) = local {
                    sc.set_config(config);
                }
            },
        );

        // ---- sport config server action ----
        let (resource_id, set_resource_id) = signal(options.object_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // server action to fetch updated sport config for the given id, used by client registry
        let fetch_sport_config = ServerAction::<LoadSportConfig>::new();
        let fetch_sport_config_pending = fetch_sport_config.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), fetch_sport_config_pending);

        let refetch = Callback::new(move |()| {
            if let Some(id) = resource_id.get() {
                fetch_sport_config.dispatch(LoadSportConfig { id });
            }
        });

        let topic = Signal::derive(move || {
            resource_id.get().map(|id| CrTopic::SportConfig {
                sport_config_id: id,
            })
        });
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // handle fetch result
        Effect::new(move || {
            if let Some(fetch_result) = fetch_sport_config.value().get() {
                fetch_sport_config.clear();
                match fetch_result {
                    Ok(Some(sc)) => {
                        set_resource_id.set(Some(sc.get_id()));
                        set_optimistic_version.set(sc.get_version());
                        local.set(Some(sc));
                    }
                    Ok(None) => {
                        // This case should not happen, since the fetch action is triggered based on the presence of a valid
                        // resource id. If it does happen, it means the resource was not found and we should inform the user.
                        let err = AppError::ResourceNotFound(
                            "Sport Config".to_string(),
                            resource_id.get().unwrap_or_default(),
                        );
                        handle_with_toast(&toast_ctx, &err, None);
                    }
                    Err(err) => {
                        let interactive = LabeledAction {
                            label: "Retry".to_string(),
                            on_click: refetch,
                        };
                        handle_with_toast(&toast_ctx, &err, Some(interactive));
                    }
                }
            }
        });

        // server action for saving the sport config based on the current state of the editor context
        let save_sport_config = ServerAction::<SaveSportConfig>::new();
        let save_sport_config_pending = save_sport_config.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), save_sport_config_pending);

        let post_save_callback = StoredValue::new(None::<Callback<SportConfig>>);

        // handle save result
        Effect::new(move || {
            if let Some(ssc_result) = save_sport_config.value().get() {
                save_sport_config.clear();
                match ssc_result {
                    Ok(sc) => {
                        set_resource_id.set(Some(sc.get_id()));
                        set_optimistic_version.set(sc.get_version());
                        local.set(Some(sc.clone()));

                        if let Some(callback) = post_save_callback.get_value() {
                            callback.run(sc);
                        }
                    }
                    Err(err) => {
                        // version reset for parallel editing
                        set_optimistic_version.set(version.get());
                        // transform unique violation error into Validation Error for name, if any
                        if let Some(object_id) = id.get()
                            && let Some(field_error) = err.to_field_error(object_id, "name")
                        {
                            set_unique_violation_error.set(Some(field_error));
                        } else {
                            handle_with_toast(&toast_ctx, &err, None);
                        }
                    }
                }
            }
        });

        SportConfigEditorContext {
            local,
            local_read_only: local.into(),
            validation_result,
            set_unique_violation_error,
            id,
            version,
            name,
            set_name,
            config,
            set_config,
            set_optimistic_version,
            save_sport_config,
            post_save_callback,
        }
    }

    /// Set an existing sport config in the editor context.
    fn set_object(&self, sc: SportConfig) {
        self.local.set(Some(sc.clone()));
        self.set_optimistic_version.set(sc.get_version());
    }

    /// Create a new sport config in the editor context with a new UUID and default values from the sport plugin.
    fn new_object(&self) -> Option<Uuid> {
        let sport_id = SportIdQuery::use_param_query();
        let state = expect_context::<Store<GlobalState>>();
        let sport_plugin_manager = state.sport_plugin_manager();
        let sport_plugin = move || {
            sport_id
                .get()
                .and_then(|id| sport_plugin_manager.get().get_web_ui(&id))
        };
        if let Some(sport_id) = sport_id.get()
            && let Some(plugin) = sport_plugin()
        {
            let id = Uuid::new_v4();
            let id_version = IdVersion::new(id, None);
            let mut sc = SportConfig::new(id_version);
            sc.set_sport_id(sport_id)
                .set_config(plugin.get_default_config());
            self.local.set(Some(sc));
            self.set_optimistic_version.set(None);
            Some(id)
        } else {
            None
        }
    }
}

impl EditorContextWithResource for SportConfigEditorContext {
    /// Get the current sport config in the editor context with its version, if any.
    fn get_versioned_object(&self) -> Option<Self::ObjectType> {
        self.local.with(|local| {
            local
                .as_ref()
                .and_then(|sc| sc.get_version().map(|_| sc.clone()))
        })
    }

    /// Create a new object from a given sport config by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, mut sc: SportConfig) -> Option<Uuid> {
        let id = Uuid::new_v4();
        sc.set_id_version(IdVersion::new(id, None)).set_name("");
        self.local.set(Some(sc));
        self.set_optimistic_version.set(None);
        Some(id)
    }

    /// Increment the optimistic version in the editor context to optimistically handle version updates after saving.
    fn increment_optimistic_version(&self) {
        self.set_optimistic_version.update(|v| {
            if let Some(current_version) = v {
                *current_version += 1
            } else {
                *v = Some(0)
            }
        });
    }

    /// If save fails, we need to reset the version to the original version to prevent version mismatch on next save attempt.
    fn reset_version_to_origin(&self) {
        self.set_optimistic_version.set(self.version.get());
    }

    /// Get the current optimistic version signal from the editor context, if any.
    fn optimistic_version_signal(&self) -> Signal<Option<u32>> {
        self.set_optimistic_version.into()
    }
}
