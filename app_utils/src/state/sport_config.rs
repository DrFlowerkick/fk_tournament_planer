//! sport configuration editor context

use crate::{
    params::{ParamQuery, SportIdQuery},
    state::{
        EditorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
    },
};
use app_core::{
    SportConfig,
    utils::{id_version::IdVersion, validation::ValidationResult},
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
    pub origin: RwSignal<Option<SportConfig>>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the id field
    pub id: Signal<Option<Uuid>>,
    /// Signal slice for the version field
    pub version: Signal<Option<u32>>,
    /// RwSignal for optimistic version handling
    set_version: StoredValue<Option<RwSignal<Option<u32>>>>,
    /// Signal slice for the name field
    pub name: Signal<Option<String>>,
    /// Callback for updating the name field
    pub set_name: Callback<Option<String>>,
    /// Signal slice for the config field
    pub config: Signal<Option<Value>>,
    /// SignalSetter for updating the config field
    pub set_config: SignalSetter<Value>,
    /// ReadSignal for checking if the current config is valid JSON
    pub is_valid_json: ReadSignal<bool>,
    /// WriteSignal for setting the is_valid_json state
    pub set_is_valid_json: WriteSignal<bool>,
}

impl EditorContext for SportConfigEditorContext {
    fn has_origin(&self) -> Signal<bool> {
        let origin = self.origin;
        Signal::derive(move || origin.with(|o| o.is_some()))
    }

    fn has_id(&self) -> Signal<bool> {
        let id = self.id;
        Signal::derive(move || id.with(|id| id.is_some()))
    }

    fn prepare_copy(&self) {
        if let Some(mut sc) = self.origin.get() {
            sc.set_id_version(IdVersion::new(Uuid::new_v4(), None))
                .set_name("");
            self.local.set(Some(sc));
            self.origin.set(None);
        }
    }

    fn new_object(&self) {
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
            let id_version = IdVersion::new(Uuid::new_v4(), None);
            let mut sc = SportConfig::new(id_version);
            sc.set_sport_id(sport_id)
                .set_config(plugin.get_default_config());
            self.local.set(Some(sc));
            self.origin.set(None);
        } else {
            self.local.set(None);
            self.origin.set(None);
        }
    }
}

impl SportConfigEditorContext {
    /// Create a new `SportConfigEditorContext`.
    pub fn new() -> Self {
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
        let validation_result = Signal::derive(move || {
            local.with(|local| {
                if let Some(sc) = local
                    && let Some(plugin) = sport_plugin()
                {
                    sc.validate(plugin)
                } else {
                    ValidationResult::Ok(())
                }
            })
        });

        let id = create_read_slice(local, move |local| local.as_ref().map(|sc| sc.get_id()));
        let version = create_read_slice(local, move |local| {
            local.as_ref().and_then(|sc| sc.get_version())
        });
        let (name, set_name) = create_slice(
            local,
            |local| local.as_ref().map(|sc| sc.get_name().to_string()),
            |local, name: String| {
                if let Some(sc) = local {
                    sc.set_name(name);
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

        let (is_valid_json, set_is_valid_json) = signal(false);

        SportConfigEditorContext {
            local,
            origin,
            is_changed,
            validation_result,
            id,
            version,
            set_version: StoredValue::new(None),
            name,
            set_name,
            config,
            set_config,
            is_valid_json,
            set_is_valid_json,
        }
    }

    pub fn set_sport_config(&self, sc: SportConfig) {
        self.local.set(Some(sc.clone()));
        leptos::logging::log!(
            "Setting local sport config in editor context: {:?}",
            self.local.get()
        );
        self.origin.set(Some(sc));
        leptos::logging::log!(
            "Setting origin sport config in editor context: {:?}",
            self.origin.get()
        );
    }

    // --- optimistic version to prevent unneeded server round after save
    /// Provide an RwSignal for the version field to optimistically handle version updates after saving.
    pub fn set_version_signal(&self, signal: Option<RwSignal<Option<u32>>>) {
        self.set_version.set_value(signal);
    }

    /// Increment the version in the editor context to optimistically handle version updates after saving.
    pub fn increment_version(&self) {
        if let Some(set_version) = self.set_version.get_value() {
            set_version.update(|version| {
                if let Some(v) = version {
                    *v += 1;
                } else {
                    *version = Some(0);
                }
            });
        }
    }

    /// If save fails, we need to reset the version to the original version to prevent version mismatch on next save attempt.
    pub fn reset_version_to_origin(&self) {
        if let Some(set_version) = self.set_version.get_value() {
            let origin_version = self
                .origin
                .with(|o| o.as_ref().and_then(|sc| sc.get_version()));
            {
                set_version.set(origin_version);
            }
        }
    }

    pub fn check_optimistic_version(&self, server_version: Option<u32>) -> bool {
        if let Some(set_version) = self.set_version.get_value() {
            let local_version = set_version.get();
            local_version == server_version
        } else {
            // If no optimistic version is set, we assume the versions are in sync.
            true
        }
    }
}
