//! sport configuration editor context

use crate::{
    params::{ParamQuery, SportIdQuery},
    state::{
        EditorContext, EditorContextWithObjectIdVersion,
        global_state::{GlobalState, GlobalStateStoreFields},
    },
};
use app_core::{
    SportConfig,
    utils::{
        id_version::IdVersion,
        validation::{FieldError, ValidationResult},
    },
};
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
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,
    /// WriteSignal for setting a unique violation error on the name field, if any
    pub set_unique_violation_error: WriteSignal<Option<FieldError>>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the id field
    pub id: Signal<Option<Uuid>>,
    /// Signal slice for the version field
    pub version: Signal<Option<u32>>,
    /// Signal for optimistic version handling to prevent unneeded server round after save
    pub optimistic_version: Signal<Option<u32>>,
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    set_optimistic_version: RwSignal<Option<u32>>,
    /// Signal slice for the name field
    pub name: Signal<Option<String>>,
    /// Callback for updating the name field
    pub set_name: Callback<Option<String>>,
    /// Signal slice for the config field
    pub config: Signal<Option<Value>>,
    /// SignalSetter for updating the config field
    pub set_config: SignalSetter<Value>,
}

impl EditorContextWithObjectIdVersion for SportConfigEditorContext {
    type ObjectTypeWithIdVersion = SportConfig;
}

impl EditorContext for SportConfigEditorContext {
    type ObjectType = SportConfig;

    /// Create a new `SportConfigEditorContext`.
    fn new() -> Self {
        let local = RwSignal::new(None::<SportConfig>);
        let origin = RwSignal::new(None);

        let is_changed = Signal::derive(move || local.get() != origin.get());

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
        let set_optimistic_version = RwSignal::new(None::<u32>);
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

        SportConfigEditorContext {
            local,
            origin,
            local_read_only: local.into(),
            is_changed,
            validation_result,
            set_unique_violation_error,
            id,
            version,
            optimistic_version: set_optimistic_version.into(),
            set_optimistic_version,
            name,
            set_name,
            config,
            set_config,
        }
    }

    /// Get the original sport config currently loaded in the editor context, if any.
    fn get_origin(&self) -> Option<Self::ObjectType> {
        self.origin.get()
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
    fn get_optimistic_version(&self) -> Signal<Option<u32>> {
        self.optimistic_version
    }
}
