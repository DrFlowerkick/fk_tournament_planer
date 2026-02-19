//! postal address editor context

use crate::state::EditorContext;
use app_core::{
    PostalAddress,
    utils::{id_version::IdVersion, validation::ValidationResult},
};
use isocountry::CountryCode;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct PostalAddressEditorContext {
    // --- state & derived signals ---
    /// The local editable postal address.
    pub local: RwSignal<Option<PostalAddress>>,
    /// The original postal address loaded from storage.
    origin: RwSignal<Option<PostalAddress>>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the postal address
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

impl EditorContext for PostalAddressEditorContext {
    fn has_origin(&self) -> Signal<bool> {
        let origin = self.origin;
        Signal::derive(move || origin.with(|o| o.is_some()))
    }

    fn has_id(&self) -> Signal<bool> {
        let id = self.id;
        Signal::derive(move || id.with(|id| id.is_some()))
    }

    fn prepare_copy(&self) {
        if let Some(mut pa) = self.origin.get() {
            pa.set_id_version(IdVersion::new(Uuid::new_v4(), None))
                .set_name("");
            self.local.set(Some(pa));
            self.origin.set(None);
        }
    }

    fn new_object(&self) {
        let id_version = IdVersion::new(Uuid::new_v4(), None);
        let pa = PostalAddress::new(id_version);

        self.local.set(Some(pa.clone()));
        self.origin.set(None);
    }
}

impl PostalAddressEditorContext {
    /// Create a new `PostalAddressEditorContext`.
    pub fn new() -> Self {
        let local = RwSignal::new(None::<PostalAddress>);
        let origin = RwSignal::new(None::<PostalAddress>);

        let is_changed = Signal::derive(move || local.get() != origin.get());
        let validation_result = Signal::derive(move || {
            local.with(|local| {
                if let Some(pa) = local {
                    pa.validate()
                } else {
                    ValidationResult::Ok(())
                }
            })
        });

        let id = create_read_slice(local, move |local| local.as_ref().map(|pa| pa.get_id()));
        let version = create_read_slice(local, move |local| {
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
            is_changed,
            validation_result,
            id,
            version,
            set_version: StoredValue::new(None),
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

    /// Set an existing postal address in the editor context.
    pub fn set_postal_address(&self, pa: PostalAddress) {
        self.local.set(Some(pa.clone()));
        self.origin.set(Some(pa));
    }

    /// Clear the postal address in the editor context.
    pub fn clear(&self) {
        self.local.set(None);
        self.origin.set(None);
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
                .with(|o| o.as_ref().and_then(|pa| pa.get_version()));
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
