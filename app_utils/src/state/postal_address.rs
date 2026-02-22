//! postal address editor context

use crate::state::{EditorContextWithObjectIdVersion, EditorContext};
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
    local: RwSignal<Option<PostalAddress>>,
    /// The original postal address loaded from storage.
    origin: RwSignal<Option<PostalAddress>>,
    /// Read slice of origin
    pub origin_read_only: Signal<Option<PostalAddress>>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the postal address
    pub validation_result: Signal<ValidationResult<()>>,

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

impl EditorContextWithObjectIdVersion for PostalAddressEditorContext {
    type ObjectTypeWithIdVersion = PostalAddress;
}

impl EditorContext for PostalAddressEditorContext {
    type ObjectType = PostalAddress;

    /// Create a new `PostalAddressEditorContext`.
    fn new() -> Self {
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
        let set_optimistic_version = RwSignal::new(None::<u32>);
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
            origin_read_only: origin.into(),
            is_changed,
            validation_result,
            id,
            version,
            optimistic_version: set_optimistic_version.into(),
            set_optimistic_version,
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

    /// Get the original postal address currently loaded in the editor context, if any.
    fn get_origin(&self) -> Option<Self::ObjectType> {
        self.origin.get()
    }

    /// Set an existing postal address in the editor context.
    fn set_object(&self, pa: PostalAddress) {
        self.local.set(Some(pa.clone()));
        self.set_optimistic_version.set(pa.get_version());
        self.origin.set(Some(pa));
    }

    /// Create a new postal address in the editor context with a new UUID and default values.
    fn new_object(&self) -> Option<Uuid> {
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, None);
        let pa = PostalAddress::new(id_version);

        self.local.set(Some(pa.clone()));
        self.set_optimistic_version.set(None);
        self.origin.set(None);
        Some(id)
    }

    /// Create a new object from a given postal address by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, mut pa: PostalAddress) -> Option<Uuid> {
        let id = Uuid::new_v4();
        pa.set_id_version(IdVersion::new(id, None)).set_name("");
        self.local.set(Some(pa));
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
