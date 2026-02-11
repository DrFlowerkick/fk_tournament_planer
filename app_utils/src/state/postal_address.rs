//! postal address editor context

use crate::hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation};
use app_core::{
    PostalAddress,
    utils::{id_version::IdVersion, validation::ValidationResult},
};
use isocountry::CountryCode;
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct PostalAddressEditorContext {
    // --- state & derived signals ---
    /// The local editable postal address.
    local: RwSignal<Option<PostalAddress>>,
    /// The original postal address loaded from storage.
    origin: StoredValue<Option<PostalAddress>>,
    /// Read slice for accessing the local postal address
    pub local_readonly: Signal<Option<PostalAddress>>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the postal address
    pub validation_result: Signal<ValidationResult<()>>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the postal_address_id field
    pub postal_address_id: Signal<Option<Uuid>>,
    /// Signal slice for the postal_address_version field
    pub postal_address_version: Signal<Option<u32>>,
    /// Signal slice for the name field
    pub name: Signal<Option<String>>,
    /// Callback for updating the name field
    pub set_name: Callback<Option<String>>,
    /// Signal slice for the street field
    pub street: Signal<Option<String>>,
    /// Callback for updating the street field
    pub set_street: Callback<Option<String>>,
    /// Signal slice for the postal_code field
    pub postal_code: Signal<Option<String>>,
    /// Callback for updating the postal_code field
    pub set_postal_code: Callback<Option<String>>,
    /// Signal slice for the locality field
    pub locality: Signal<Option<String>>,
    /// Callback for updating the locality field
    pub set_locality: Callback<Option<String>>,
    /// Signal slice for the region field
    pub region: Signal<Option<String>>,
    /// Callback for updating the region field
    pub set_region: Callback<Option<String>>,
    /// Signal slice for the country field
    pub country: Signal<Option<CountryCode>>,
    /// Callback for updating the country field
    pub set_country: Callback<Option<CountryCode>>,
}

impl PostalAddressEditorContext {
    /// Create a new `PostalAddressEditorContext`.
    pub fn new() -> Self {
        let local = RwSignal::new(None::<PostalAddress>);
        let origin = StoredValue::new(None);

        let is_changed = Signal::derive(move || local.get() != origin.get_value());
        let validation_result = Signal::derive(move || {
            local.with(|local| {
                if let Some(pa) = local {
                    pa.validate()
                } else {
                    ValidationResult::Ok(())
                }
            })
        });

        let postal_address_id =
            create_read_slice(local, move |local| local.as_ref().map(|pa| pa.get_id()));

        let postal_address_version = create_read_slice(local, move |local| {
            local.as_ref().and_then(|pa| pa.get_version())
        });
        let (name, set_name) = create_slice(
            local,
            |local| local.as_ref().map(|pa| pa.get_name().to_string()),
            |local, name: String| {
                if let Some(pa) = local {
                    pa.set_name(name);
                }
            },
        );
        let set_name = Callback::new(move |name: Option<String>| {
            set_name.set(name.unwrap_or_default());
        });
        let (street, set_street) = create_slice(
            local,
            |local| local.as_ref().map(|pa| pa.get_street().to_string()),
            |local, street: String| {
                if let Some(pa) = local {
                    pa.set_street(street);
                }
            },
        );
        let set_street = Callback::new(move |street: Option<String>| {
            set_street.set(street.unwrap_or_default());
        });
        let (postal_code, set_postal_code) = create_slice(
            local,
            |local| local.as_ref().map(|pa| pa.get_postal_code().to_string()),
            |local, postal_code: String| {
                if let Some(pa) = local {
                    pa.set_postal_code(postal_code);
                }
            },
        );
        let set_postal_code = Callback::new(move |postal_code: Option<String>| {
            set_postal_code.set(postal_code.unwrap_or_default());
        });
        let (locality, set_locality) = create_slice(
            local,
            |local| local.as_ref().map(|pa| pa.get_locality().to_string()),
            |local, locality: String| {
                if let Some(pa) = local {
                    pa.set_locality(locality);
                }
            },
        );
        let set_locality = Callback::new(move |locality: Option<String>| {
            set_locality.set(locality.unwrap_or_default());
        });
        let (region, set_region) = create_slice(
            local,
            |local| {
                local
                    .as_ref()
                    .and_then(|pa| pa.get_region().map(|r| r.to_string()))
            },
            |local, region: String| {
                if let Some(pa) = local {
                    pa.set_region(region);
                }
            },
        );
        let set_region = Callback::new(move |region: Option<String>| {
            set_region.set(region.unwrap_or_default());
        });
        let (country, set_country) = create_slice(
            local,
            |local| local.as_ref().and_then(|pa| pa.get_country()),
            |local, country: Option<CountryCode>| {
                if let Some(pa) = local {
                    pa.set_country(country);
                }
            },
        );
        let set_country = Callback::new(move |country: Option<CountryCode>| {
            set_country.set(country);
        });
        PostalAddressEditorContext {
            local,
            origin,
            local_readonly: local.read_only().into(),
            is_changed,
            validation_result,
            postal_address_id,
            postal_address_version,
            name,
            set_name,
            street,
            set_street,
            postal_code,
            set_postal_code,
            locality,
            set_locality,
            region,
            set_region,
            country,
            set_country,
        }
    }

    /// Create a new postal address in the editor context.
    pub fn new_postal_address(&self) {
        let id_version = IdVersion::new(Uuid::new_v4(), None);
        let pa = PostalAddress::new(id_version);

        self.local.set(Some(pa.clone()));
        self.origin.set_value(None);
    }

    /// Set an existing postal address in the editor context.
    pub fn set_postal_address(&self, pa: PostalAddress) {
        self.local.set(Some(pa.clone()));
        self.origin.set_value(Some(pa));
    }
}

#[derive(Clone, Copy)]
pub struct PostalAddressListContext {
    /// Read slice for the currently selected postal address id
    pub selected_id: Signal<Option<Uuid>>,
    /// Callback for updating the currently selected postal address id
    pub set_selected_id: Callback<Option<Uuid>>,
    /// Trigger to refetch data from server
    refetch_trigger: RwSignal<u64>,
    /// Read slice for getting the current state of the postal address list
    pub track_fetch_trigger: Signal<u64>,
}

impl PostalAddressListContext {
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
                    url_update_query("address_id", &t_id.to_string())
                } else {
                    url_remove_query("address_id")
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
