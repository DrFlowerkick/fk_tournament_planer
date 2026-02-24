//! sport configuration editor context

use crate::{
    error::{
        AppError, AppResult, map_db_unique_violation_to_field_error, strategy::handle_write_error,
    },
    params::{ParamQuery, SportIdQuery},
    server_fn::sport_config::{SaveSportConfig, load_sport_config},
    state::{
        EditorContext, EditorContextWithResource,
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
    /// The original sport configuration loaded from storage.
    origin: RwSignal<Option<SportConfig>>,
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
    /// Resource for loading the sport configuration based on the given id in the editor options
    pub load_sport_config: LocalResource<AppResult<Option<SportConfig>>>,
    /// Server action for saving the sport configuration based on the current state of the editor context
    pub save_sport_config: ServerAction<SaveSportConfig>,
    /// Callback after successful save to e.g. navigate to the new sport configuration or show a success toast.
    pub post_save_callback: StoredValue<Option<Callback<SportConfig>>>,
}

impl EditorContext for SportConfigEditorContext {
    type ObjectType = SportConfig;
    type NewEditorOptions = Option<Uuid>;

    /// Create a new `SportConfigEditorContext`.
    fn new(res_id: Option<Uuid>) -> Self {
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
        let origin = RwSignal::new(None);

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

        // ---- sport config resource ----
        let (resource_id, set_resource_id) = signal(res_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // resource to load sport config
        // since we render SportConfigTableRow inside the Transition block of ListSportConfigs,
        // we do not need to use another Transition block to load the sport config.
        /*let load_sport_config = Resource::new(
            move || resource_id.get(),
            move |maybe_id| async move {
                if let Some(id) = maybe_id {
                    match activity_tracker
                        .track_activity_wrapper(component_id.get_value(), load_sport_config(id))
                        .await
                    {
                        Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                        res =>res,
                    }
                } else {
                    Ok(None)
                }
            },
        );*/
        // At current state of leptos SSR does not provide stable rendering (meaning during initial load Hydration
        // errors occur until the page is fully rendered and the app "transformed" into a SPA). For this reason
        // we use a LocalResource here, which does not cause hydration errors.
        // ToDo: investigate how to use Resource without hydration errors, since Resource provides better
        // ergonomics for loading states and error handling.
        let load_sport_config = LocalResource::new(move || async move {
            if let Some(id) = resource_id.get() {
                match activity_tracker
                    .track_activity_wrapper(component_id.get_value(), load_sport_config(id))
                    .await
                {
                    Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                    res => res,
                }
            } else {
                Ok(None)
            }
        });

        let topic = Signal::derive(move || resource_id.get().map(|id| CrTopic::SportConfig(id)));
        let refetch = Callback::new(move |()| {
            load_sport_config.refetch();
        });
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // ---- sport config server action ----
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
                        origin.set(Some(sc.clone()));

                        if let Some(callback) = post_save_callback.get_value() {
                            callback.run(sc);
                        }
                    }
                    Err(err) => {
                        // version reset for parallel editing
                        set_optimistic_version.set(version.get());
                        // transform unique violation error into Validation Error for name, if any
                        if let Some(object_id) = id.get()
                            && let Some(field_error) =
                                map_db_unique_violation_to_field_error(&err, object_id, "name")
                        {
                            set_unique_violation_error.set(Some(field_error));
                        } else {
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
            }
        });

        SportConfigEditorContext {
            local,
            origin,
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
            load_sport_config,
            save_sport_config,
            post_save_callback,
        }
    }

    /// Get the original sport config currently loaded in the editor context, if any.
    fn origin_signal(&self) -> Signal<Option<Self::ObjectType>> {
        self.origin.into()
    }

    /// Set an existing sport config in the editor context.
    fn set_object(&self, sc: SportConfig) {
        self.local.set(Some(sc.clone()));
        self.set_optimistic_version.set(sc.get_version());
        self.origin.set(Some(sc));
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
            self.origin.set(None);
            Some(id)
        } else {
            None
        }
    }
}

impl EditorContextWithResource for SportConfigEditorContext {
    /// Create a new object from a given sport config by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, mut sc: SportConfig) -> Option<Uuid> {
        let id = Uuid::new_v4();
        sc.set_id_version(IdVersion::new(id, None)).set_name("");
        self.local.set(Some(sc));
        self.set_optimistic_version.set(None);
        self.origin.set(None);
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
