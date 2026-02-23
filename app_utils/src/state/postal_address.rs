//! postal address editor context

use crate::{
    error::{
        AppError, AppResult, map_db_unique_violation_to_field_error, strategy::handle_write_error,
    },
    server_fn::postal_address::{SavePostalAddress, load_postal_address},
    state::{
        EditorContext, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        toast_state::ToastContext,
    },
};
use app_core::{
    CrTopic, PostalAddress,
    utils::{
        id_version::IdVersion,
        validation::{FieldError, ValidationResult},
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
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
    /// Read slice for accessing the validation result of the postal address
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

    // --- Resource & server action state ---
    /// Resource for loading the postal address based on the given id in the editor options
    pub load_postal_address: LocalResource<AppResult<Option<PostalAddress>>>,
    /// Server action for saving the postal address based on the current state of the editor context
    pub save_postal_address: ServerAction<SavePostalAddress>,
    /// Callback after successful save to e.g. navigate to the new postal address or show a success toast.
    pub post_save_callback: StoredValue<Option<Callback<PostalAddress>>>,
}

impl EditorContext for PostalAddressEditorContext {
    type ObjectType = PostalAddress;
    type NewEditorOptions = Option<Uuid>;

    /// Create a new `PostalAddressEditorContext`.
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
        let local = RwSignal::new(None::<PostalAddress>);
        let origin = RwSignal::new(None::<PostalAddress>);
        let (unique_violation_error, set_unique_violation_error) = signal(None::<FieldError>);
        let validation_result = Signal::derive(move || {
            let vr = local.with(|local| {
                if let Some(pa) = local {
                    pa.validate()
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

        let id = create_read_slice(local, move |local| local.as_ref().map(|pa| pa.get_id()));
        let version = create_read_slice(local, move |local| {
            local.as_ref().and_then(|pa| pa.get_version())
        });
        let (name, set_name) = create_slice(
            local,
            |local| local.as_ref().map(|pa| pa.get_name().to_string()),
            move |local, name: String| {
                if let Some(pa) = local {
                    pa.set_name(name);
                    // Clear unique violation error on name change, if any
                    set_unique_violation_error.set(None);
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

        // ---- address resource ----
        let (resource_id, set_resource_id) = signal(res_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // resource to load postal address
        // since we render PostalAddressTableRow inside the Transition block of ListPostalAddresses,
        // we do not need to use another Transition block to load the postal address.
        /*let address_res = Resource::new(
            move || resource_id.get(),
            move |maybe_id| async move {
                if let Some(id) = maybe_id {
                    match activity_tracker
                        .track_activity_wrapper(component_id.get_value(), load_postal_address(id))
                        .await
                    {
                        Ok(None) => {
                            Err(AppError::ResourceNotFound("Postal Address".to_string(), id))
                        }
                        res => res,
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
        let load_postal_address = LocalResource::new(move || async move {
            if let Some(id) = resource_id.get() {
                match activity_tracker
                    .track_activity_wrapper(component_id.get_value(), load_postal_address(id))
                    .await
                {
                    Ok(None) => Err(AppError::ResourceNotFound("Postal Address".to_string(), id)),
                    res => res,
                }
            } else {
                Ok(None)
            }
        });

        let topic = Signal::derive(move || resource_id.get().map(|id| CrTopic::Address(id)));
        let refetch = Callback::new(move |()| {
            load_postal_address.refetch();
        });
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // ---- address server action ----
        let save_postal_address = ServerAction::<SavePostalAddress>::new();
        let save_postal_address_pending = save_postal_address.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), save_postal_address_pending);

        let post_save_callback = StoredValue::new(None::<Callback<PostalAddress>>);

        // ToDo: with auto save and parallel editing, refetch is done automatically. Delete this dummy refetch.
        let refetch = Callback::new(move |_| {});

        // handle save result
        Effect::new(move || {
            if let Some(spa_result) = save_postal_address.value().get() {
                save_postal_address.clear();
                match spa_result {
                    Ok(pa) => {
                        set_resource_id.set(Some(pa.get_id()));
                        set_optimistic_version.set(pa.get_version());
                        local.set(Some(pa.clone()));
                        origin.set(Some(pa.clone()));

                        if let Some(callback) = post_save_callback.get_value() {
                            callback.run(pa);
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

        PostalAddressEditorContext {
            local,
            origin,
            origin_read_only: origin.into(),
            validation_result,
            set_unique_violation_error,
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
            load_postal_address,
            save_postal_address,
            post_save_callback,
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
