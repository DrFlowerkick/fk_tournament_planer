//! Postal Address Edit Module

use app_core::PostalAddress;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::save_postal_address_inner;
use app_utils::{
    components::inputs::{EnumSelectWithValidation, TextInputWithValidation},
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error, handle_write_error},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::use_address_id_query,
    server_fn::postal_address::{SavePostalAddress, load_postal_address},
    state::{
        error_state::PageErrorContext,
        postal_address::{PostalAddressEditorContext, PostalAddressListContext},
        toast_state::ToastContext,
    },
};
use leptos::{html::H2, prelude::*};
#[cfg(feature = "test-mock")]
use leptos::{wasm_bindgen::JsCast, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_matched, use_navigate},
};
use uuid::Uuid;

#[component]
pub fn LoadPostalAddress() -> impl IntoView {
    // --- global state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // --- Server Resources ---
    let postal_address_id = use_address_id_query();
    let addr_res = Resource::new(
        move || postal_address_id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_postal_address(id).await {
                    Ok(None) => Err(AppError::ResourceNotFound("Postal Address".to_string(), id)),
                    load_result => load_result,
                },
                None => Ok(None),
            }
        },
    );

    let refetch = Callback::new(move |()| {
        addr_res.refetch();
    });

    let on_cancel = use_on_cancel();

    view! {
        <Transition fallback=move || {
            view! {
                <div class="flex justify-center items-center p-4">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            }
        }>
            <ErrorBoundary fallback=move |errors| {
                for (_err_id, err) in errors.get().into_iter() {
                    let e = err.into_inner();
                    if let Some(app_err) = e.downcast_ref::<AppError>() {
                        handle_read_error(
                            &page_err_ctx,
                            component_id.get_value(),
                            app_err,
                            refetch,
                            on_cancel,
                        );
                    } else {
                        handle_general_error(
                            &page_err_ctx,
                            component_id.get_value(),
                            "An unexpected error occurred.",
                            None,
                            on_cancel,
                        );
                    }
                }
            }>

                {move || {
                    addr_res
                        .and_then(|may_be_pa| {
                            view! {
                                <EditPostalAddress
                                    postal_address=may_be_pa.clone()
                                    refetch=refetch.clone()
                                />
                            }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn EditPostalAddress(
    postal_address: Option<PostalAddress>,
    refetch: Callback<()>,
) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_matched_route_update_query,
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let matched_route = use_matched();

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    let postal_address_list_ctx = expect_context::<PostalAddressListContext>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    let postal_address_editor = PostalAddressEditorContext::new();
    let (show_form, is_new) = if let Some(pa) = postal_address {
        postal_address_editor.set_postal_address(pa);
        (true, false)
    } else {
        postal_address_editor.new_postal_address();
        let is_new = matched_route.get_untracked().ends_with("new");
        (is_new, is_new)
    };
    provide_context(postal_address_editor);

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // --- Server Actions ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();

    // handle save result
    Effect::new(move || {
        match save_postal_address.value().get() {
            Some(Ok(pa)) => {
                save_postal_address.clear();
                toast_ctx.success("Postal Address saved successfully");
                if is_new {
                    postal_address_list_ctx.trigger_refetch();
                }
                let nav_url = url_matched_route_update_query(
                    "address_id",
                    &pa.get_id().to_string(),
                    MatchedRouteHandler::RemoveSegment(1),
                );
                navigate(&nav_url, NavigateOptions::default());
            }
            Some(Err(err)) => {
                leptos::logging::log!("Error saving Postal Address: {:?}", err);
                save_postal_address.clear();
                handle_write_error(
                    &page_err_ctx,
                    &toast_ctx,
                    component_id.get_value(),
                    &err,
                    refetch,
                );
            }
            None => { /* saving state - do nothing */ }
        }
    });

    let save_postal_address_pending = save_postal_address.pending();

    // --- Signals for UI state & errors ---
    // use try, because these signals are use in conjunction with page_err_ctx,
    // which has another "lifetime" in the reactive system, which may cause panics
    // for the other signals when the component is unmounted.
    let is_disabled =
        move || save_postal_address_pending.try_get().unwrap_or(false) || page_err_ctx.has_errors();

    let is_valid_addr = move || {
        postal_address_editor
            .validation_result
            .try_with(|vr| vr.is_ok())
            .unwrap_or(false)
    };

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    {move || { if is_new { "New Postal Address" } else { "Edit Postal Address" } }}
                </h2>
                <Show
                    when=move || show_form
                    fallback=|| {
                        view! {
                            <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                <p class="text-2xl font-bold text-center">"Please select a postal address from the list."</p>
                            </div>
                        }
                    }
                >
                // --- Address Form ---
                <div data-testid="form-address">
                    <ActionForm
                        action=save_postal_address
                        on:submit:capture=move |ev| {
                            #[cfg(feature = "test-mock")]
                            {
                                ev.prevent_default();
                                let intent = ev
                                    .submitter()
                                    .and_then(|el| {
                                        el.dyn_into::<web_sys::HtmlButtonElement>().ok()
                                    })
                                    .map(|btn| btn.value());
                                let data = SavePostalAddress {
                                    id: postal_address_editor
                                        .postal_address_id
                                        .get()
                                        .unwrap_or(Uuid::nil()),
                                    version: postal_address_editor
                                        .local_readonly
                                        .get()
                                        .map_or(0, |pa| pa.get_version().unwrap_or_default()),
                                    name: postal_address_editor.name.get().unwrap_or_default(),
                                    street: postal_address_editor.street.get().unwrap_or_default(),
                                    postal_code: postal_address_editor
                                        .postal_code
                                        .get()
                                        .unwrap_or_default(),
                                    locality: postal_address_editor
                                        .locality
                                        .get()
                                        .unwrap_or_default(),
                                    region: postal_address_editor.region.get(),
                                    country: postal_address_editor
                                        .country
                                        .get()
                                        .map(|c| c.alpha2().to_string())
                                        .unwrap_or_default(),
                                    intent,
                                };
                                let save_action = Action::new(|pa: &SavePostalAddress| {
                                    let pa = pa.clone();
                                    async move {
                                        save_postal_address_inner(
                                                pa.id,
                                                pa.version,
                                                pa.name,
                                                pa.street,
                                                pa.postal_code,
                                                pa.locality,
                                                pa.region,
                                                pa.country,
                                                pa.intent,
                                            )
                                            .await
                                    }
                                });
                                save_action.dispatch(data);
                            }
                            #[cfg(not(feature = "test-mock"))]
                            {
                                let _ = ev;
                            }
                        }
                    >
                        // --- Address Form Fields ---
                        <fieldset class="space-y-4" prop:disabled=is_disabled>
                            // Hidden meta fields the server expects (id / version)
                            <input
                                type="hidden"
                                name="id"
                                data-testid="hidden-id"
                                prop:value=move || {
                                    postal_address_editor
                                        .postal_address_id
                                        .get()
                                        .unwrap_or(Uuid::nil())
                                        .to_string()
                                }
                            />
                            <input
                                type="hidden"
                                name="version"
                                data-testid="hidden-version"
                                prop:value=move || {
                                    postal_address_editor
                                        .local_readonly
                                        .get()
                                        .map_or(0, |pa| pa.get_version().unwrap_or_default())
                                }
                            />
                            <TextInputWithValidation
                                label="Name"
                                name="name"
                                value=postal_address_editor.name
                                set_value=postal_address_editor.set_name
                                validation_result=postal_address_editor.validation_result
                                object_id=postal_address_editor.postal_address_id
                                field="Name"
                            />
                            <TextInputWithValidation
                                label="Street & number"
                                name="street"
                                value=postal_address_editor.street
                                set_value=postal_address_editor.set_street
                                validation_result=postal_address_editor.validation_result
                                object_id=postal_address_editor.postal_address_id
                                field="Street"
                            />
                            <div class="grid grid-cols-2 gap-4">
                                <TextInputWithValidation
                                    label="Postal code"
                                    name="postal_code"
                                    value=postal_address_editor.postal_code
                                    set_value=postal_address_editor.set_postal_code
                                    validation_result=postal_address_editor.validation_result
                                    object_id=postal_address_editor.postal_address_id
                                    field="PostalCode"
                                />
                                <TextInputWithValidation
                                    label="City"
                                    name="locality"
                                    value=postal_address_editor.locality
                                    set_value=postal_address_editor.set_locality
                                    validation_result=postal_address_editor.validation_result
                                    object_id=postal_address_editor.postal_address_id
                                    field="Locality"
                                />
                            </div>
                            <TextInputWithValidation
                                label="Region"
                                name="region"
                                value=postal_address_editor.region
                                set_value=postal_address_editor.set_region
                                optional=true
                            />
                            <EnumSelectWithValidation
                                label="Country"
                                name="country"
                                value=postal_address_editor.country
                                set_value=postal_address_editor.set_country
                                validation_result=postal_address_editor.validation_result
                                object_id=postal_address_editor.postal_address_id
                                field="Country"
                            />
                            <div class="card-actions justify-end mt-4">
                                <button
                                    type="submit"
                                    name="intent"
                                    value=move || if is_new { "create" } else { "update" }
                                    data-testid="btn-save"
                                    class="btn btn-primary"
                                    prop:disabled=move || is_disabled() || !is_valid_addr()
                                >
                                    "Save"
                                </button>

                                <button
                                    type="submit"
                                    name="intent"
                                    value="create"
                                    data-testid="btn-save-as-new"
                                    class="btn btn-secondary"
                                    prop:disabled=move || {
                                        is_disabled() || is_new || !is_valid_addr()
                                    }
                                    prop:hidden=move || is_new
                                >
                                    "Save as new"
                                </button>

                                <button
                                    type="button"
                                    name="intent"
                                    value="cancel"
                                    data-testid="btn-cancel"
                                    class="btn btn-ghost"
                                    on:click=move |_| on_cancel.run(())
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </fieldset>
                    </ActionForm>
                </div>
                </Show>
            </div>
        </div>
    }
}
