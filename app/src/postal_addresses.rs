// web ui for adding and modifying postal addresses

use crate::AppResult;
use app_core::PostalAddress;
use leptos::{logging::log, prelude::*};
use leptos_router::{hooks::use_params, params::Params};
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
struct AddressParams {
    pub uuid: Option<Uuid>,
}

#[component]
pub fn new_postal_address() -> impl IntoView {
    view! { <AddressFormWrapper id=None /> }
}

#[component]
pub fn view_postal_address() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let id = params.get().map(|ap| ap.uuid).unwrap_or(None);
    log!("{:?}", id);
    view! { <AddressFormWrapper id=id /> }
}

#[component]
pub fn AddressFormWrapper(id: Option<Uuid>) -> impl IntoView {
    // Resource: load existing address when `id` is Some(...)
    let addr_res = Resource::new(
        move || id,
        |maybe_id| async move {
            match maybe_id {
                // AppResult<PostalAddress>
                Some(id) => load_postal_address(id).await,
                // new form: no loading delay
                None => Ok(Default::default()),
            }
        },
    );

    view! {
        <Suspense fallback=move || {
            view! { <AddressForm address=PostalAddress::default() loading=true /> }
        }>
            {move || {
                addr_res
                    .get()
                    .map(|a| view! { <AddressForm address=a.unwrap_or_default() loading=false /> })
            }}
        </Suspense>
    }
}

#[component]
pub fn AddressForm(address: PostalAddress, loading: bool) -> impl IntoView {

    let save_postal_address = ServerAction::<SavePostalAddress>::new();

    let is_new = address.version == -1 || address.id.is_nil();

    view! {
        // Use <ActionForm/> to bind to your save server fn
        <ActionForm action=save_postal_address>
            // Hidden meta fields the server expects (id / version / intent)
            <input type="hidden" name="id" prop:value=address.id.to_string() />
            <input type="hidden" name="version" prop:value=address.version />

            // Disable the whole form while loading existing data
            <fieldset prop:disabled=move || loading>
                // Example: Name
                <label class="block">
                    <span class="block text-sm">"Name (optional)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="name"
                        // show live value when loaded; while loading show "Loading…" as placeholder
                        prop:value=address.name.unwrap_or_default()
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter name (optional)..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Street
                <label class="block">
                    <span class="block text-sm">"Street & number"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="street_address"
                        prop:value=address.street_address
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter street and number..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Postal code + City
                <div class="grid grid-cols-2 gap-3">
                    <label class="block">
                        <span class="block text-sm">"Postal code"</span>
                        <input
                            class="w-full border rounded p-2"
                            name="postal_code"
                            prop:value=address.postal_code
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter postal code..."
                                } else {
                                    ""
                                }
                            }
                        />
                    </label>
                    <label class="block">
                        <span class="block text-sm">"City"</span>
                        <input
                            class="w-full border rounded p-2"
                            name="address_locality"
                            prop:value=address.address_locality
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter city..."
                                } else {
                                    ""
                                }
                            }
                        />
                    </label>
                </div>

                // Region (optional)
                <label class="block">
                    <span class="block text-sm">"Region (optional)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="address_region"
                        prop:value=address.address_region
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter region (optional)..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Country
                <label class="block">
                    <span class="block text-sm">"Country (ISO/name)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="address_country"
                        prop:value=address.address_country
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter country..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Actions: Update vs. "Save as new"
                <div class="flex gap-2">
                    // Update existing (disabled in "new" mode)
                    <button
                        type="submit"
                        name="intent"
                        value="update"
                        class="rounded px-4 py-2 border"
                        prop:disabled=move || loading || is_new
                        prop:hidden=move || is_new
                    >
                        "Save"
                    </button>

                    // Always allow "save as new"
                    <button
                        type="submit"
                        name="intent"
                        value="create"
                        class="rounded px-4 py-2 border"
                        prop:disabled=move || loading
                    >
                        "Save as new"
                    </button>
                </div>
            </fieldset>
        </ActionForm>
    }
}

#[server]
pub async fn load_postal_address(id: Uuid) -> AppResult<PostalAddress> {
    // get core from context
    use app_core::CoreState;
    let mut core = expect_context::<CoreState>().as_postal_address_state();
    let pa = if let Some(pa) = core.load(id).await? {
        log!("{:?}", pa);
        pa.to_owned()
    } else {
        PostalAddress::default()
    };
    Ok(pa)
}

#[server]
pub async fn save_postal_address(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form; -1 => new; else => update
    version: i64,
    // optional text field: treat "" as None
    name: Option<String>,
    street_address: String,
    postal_code: String,
    address_locality: String,
    // optional text field: treat "" as None
    address_region: Option<String>,
    address_country: String,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<()> {
    // get core from context
    use app_core::CoreState;
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    if matches!(intent.as_deref(), Some("update")) {
        // set id and version previously loaded
        core.set_id(id);
        core.set_version(version);
    }

    let name = name.unwrap_or_default();
    core.change_name(name);
    core.change_street_address(street_address);
    core.change_postal_code(postal_code);
    core.change_address_locality(address_locality);
    let address_region = address_region.unwrap_or_default();
    core.change_address_region(address_region);
    core.change_address_country(address_country);

    // ToDo: gracefully handle errors, e.g. retry
    let saved = core.save().await?;
    log!("{}", saved.id);
    let route = format!("/postal_address/{}", saved.id);
    // redirect to newly saved postal address
    leptos_axum::redirect(&route);
    Ok(())
}
