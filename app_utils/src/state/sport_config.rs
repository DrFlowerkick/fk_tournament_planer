//! sport configuration editor context

use crate::hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation};
use app_core::{
    SportConfig,
    utils::{id_version::IdVersion, validation::ValidationResult},
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct SportConfigEditorContext {
    // --- state & derived signals ---
    /// The local editable sport configuration.
    local: RwSignal<Option<SportConfig>>,
    /// The original sport configuration loaded from storage.
    origin: StoredValue<Option<SportConfig>>,
    /// Read slice for accessing the local sport configuration
    pub local_readonly: Signal<Option<SportConfig>>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the sport_config_id field
    pub sport_config_id: Signal<Option<Uuid>>,
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

impl SportConfigEditorContext {
    /// Create a new `SportConfigEditorContext`.
    pub fn new() -> Self {
        let local = RwSignal::new(None::<SportConfig>);
        let origin = StoredValue::new(None);

        let is_changed = Signal::derive(move || local.get() != origin.get_value());
        let validation_result = Signal::derive(move || {
            local.with(|local| {
                if let Some(sc) = local {
                    sc.validate()
                } else {
                    ValidationResult::Ok(())
                }
            })
        });

        let sport_config_id =
            create_read_slice(local, move |local| local.as_ref().map(|sc| sc.get_id()));

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
            local_readonly: local.read_only().into(),
            is_changed,
            validation_result,
            sport_config_id,
            name,
            set_name,
            config,
            set_config,
            is_valid_json,
            set_is_valid_json,
        }
    }

    pub fn new_sport_config(&self, sport_id: Uuid, default_config: Value) {
        let id_version = IdVersion::new(Uuid::new_v4(), None);
        let mut sc = SportConfig::new(id_version);
        sc.set_sport_id(sport_id).set_config(default_config);

        self.local.set(Some(sc));
        self.origin.set_value(None);
    }

    pub fn set_sport_config(&self, sc: SportConfig) {
        self.local.set(Some(sc.clone()));
        self.origin.set_value(Some(sc));
    }
}

#[derive(Clone, Copy)]
pub struct SportConfigListContext {
    /// Read slice for the currently selected sport config id
    pub selected_id: Signal<Option<Uuid>>,
    /// Callback for updating the currently selected sport config id
    pub set_selected_id: Callback<Option<Uuid>>,
    /// Trigger to refetch data from server
    refetch_trigger: RwSignal<u64>,
    /// Read slice for getting the current state of the tournament editor
    pub track_fetch_trigger: Signal<u64>,
}

impl SportConfigListContext {
    pub fn new() -> Self {
        let UseQueryNavigationReturn {
            url_update_query,
            url_remove_query,
            ..
        } = use_query_navigation();
        let navigate = use_navigate();

        let selected_id = RwSignal::new(None);
        let set_selected_id = Callback::new({
            let navigate = navigate.clone();
            move |new_id: Option<Uuid>| {
                selected_id.set(new_id);

                let nav_url = if let Some(t_id) = new_id {
                    url_update_query("sport_config_id", &t_id.to_string())
                } else {
                    url_remove_query("sport_config_id")
                };
                navigate(
                    &nav_url,
                    NavigateOptions {
                        replace: true,
                        scroll: false,
                        ..Default::default()
                    },
                );
            }
        });
        let refetch_trigger = RwSignal::new(0);

        Self {
            selected_id: selected_id.read_only().into(),
            set_selected_id,
            refetch_trigger,
            track_fetch_trigger: refetch_trigger.read_only().into(),
        }
    }

    pub fn trigger_refetch(&self) {
        self.refetch_trigger.update(|v| *v += 1);
    }
}
