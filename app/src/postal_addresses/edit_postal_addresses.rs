//! Postal Address Edit Module

use app_core::PostalAddress;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::{SavePostalAddressFormData, save_postal_address_inner};
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
    params::{AddressIdQuery, FilterNameQuery, ParamQuery},
    server_fn::postal_address::{SavePostalAddress, load_postal_address},
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table_list::ObjectListContext, postal_address::PostalAddressEditorContext,
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
    let activity_tracker = expect_context::<ActivityTracker>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- Server Resources ---
    let postal_address_id = AddressIdQuery::use_param_query();
    let addr_res = Resource::new(
        move || postal_address_id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match activity_tracker
                    .track_activity_wrapper(component_id.get_value(), load_postal_address(id))
                    .await
                {
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
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="flex justify-center items-center p-4">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    </div>
                </div>
            }
        }>
            <ErrorBoundary fallback=move |errors| {
                for (_err_id, err) in errors.get().into_iter() {
                    let e = err.into_inner();
                    leptos::logging::log!("Error saving Postal Address: {:?}", e);
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
                                    refetch=refetch
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
        get_query,
        url_matched_route_update_query,
        url_matched_route_update_queries,
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let matched_route = use_matched();

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    let activity_tracker = expect_context::<ActivityTracker>();

    let postal_address_list_ctx =
        expect_context::<ObjectListContext<PostalAddress, AddressIdQuery>>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
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
    let save_postal_address_pending = save_postal_address.pending();
    activity_tracker.track_pending_memo(component_id.get_value(), save_postal_address_pending);

    // handle save result
    Effect::new(move || {
        match save_postal_address.value().get() {
            Some(Ok(pa)) => {
                let pa_id = pa.get_id();
                save_postal_address.clear();
                toast_ctx.success("Postal Address saved successfully");
                if postal_address_list_ctx.is_id_in_list(pa_id) {
                    let nav_url = url_matched_route_update_query(
                        AddressIdQuery::key(),
                        &pa_id.to_string(),
                        MatchedRouteHandler::RemoveSegment(1),
                    );
                    navigate(&nav_url, NavigateOptions::default());
                    postal_address_list_ctx.trigger_refetch();
                } else {
                    let refetch =
                        get_query(FilterNameQuery::key()) != Some(pa.get_name().to_string());
                    let pa_id = pa_id.to_string();
                    let key_value = vec![
                        (AddressIdQuery::key(), pa_id.as_str()),
                        (FilterNameQuery::key(), pa.get_name()),
                    ];
                    let nav_url = url_matched_route_update_queries(
                        key_value,
                        MatchedRouteHandler::RemoveSegment(1),
                    );
                    navigate(&nav_url, NavigateOptions::default());
                    if refetch {
                        postal_address_list_ctx.trigger_refetch();
                    }
                }
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

    // --- Signals for UI state & errors ---
    let is_disabled = move || save_postal_address_pending.get();

    let is_valid_addr = move || {
        postal_address_editor
            .validation_result
            .with(|vr| vr.is_ok())
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
                                <p class="text-2xl font-bold text-center">
                                    "Please select a postal address from the list."
                                </p>
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
                                        form: SavePostalAddressFormData {
                                            id: postal_address_editor
                                                .postal_address_id
                                                .get()
                                                .unwrap_or(Uuid::nil()),
                                            version: postal_address_editor
                                                .postal_address_version
                                                .get()
                                                .unwrap_or_default(),
                                            name: postal_address_editor.name.get().unwrap_or_default(),
                                            street: postal_address_editor
                                                .street
                                                .get()
                                                .unwrap_or_default(),
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
                                        },
                                    };
                                    let save_action = Action::new(|pa: &SavePostalAddress| {
                                        let pa = pa.clone();
                                        async move {
                                            let result = save_postal_address_inner(pa.form).await;
                                            leptos::web_sys::console::log_1(
                                                &format!("Result of save postal address: {:?}", result)
                                                    .into(),
                                            );
                                            result
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
                            <fieldset class="space-y-4 contents" prop:disabled=is_disabled>
                                // Hidden meta fields the server expects (id / version)
                                <input
                                    type="hidden"
                                    name="form[id]"
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
                                    name="form[version]"
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
                                    name="form[name]"
                                    data_testid="input-name"
                                    value=postal_address_editor.name
                                    set_value=postal_address_editor.set_name
                                    validation_result=postal_address_editor.validation_result
                                    object_id=postal_address_editor.postal_address_id
                                    field="Name"
                                />
                                <TextInputWithValidation
                                    label="Street & number"
                                    name="form[street]"
                                    data_testid="input-street"
                                    value=postal_address_editor.street
                                    set_value=postal_address_editor.set_street
                                    validation_result=postal_address_editor.validation_result
                                    object_id=postal_address_editor.postal_address_id
                                    field="Street"
                                />
                                <div class="grid grid-cols-2 gap-4">
                                    <TextInputWithValidation
                                        label="Postal code"
                                        name="form[postal_code]"
                                        data_testid="input-postal_code"
                                        value=postal_address_editor.postal_code
                                        set_value=postal_address_editor.set_postal_code
                                        validation_result=postal_address_editor.validation_result
                                        object_id=postal_address_editor.postal_address_id
                                        field="PostalCode"
                                    />
                                    <TextInputWithValidation
                                        label="City"
                                        name="form[locality]"
                                        data_testid="input-locality"
                                        value=postal_address_editor.locality
                                        set_value=postal_address_editor.set_locality
                                        validation_result=postal_address_editor.validation_result
                                        object_id=postal_address_editor.postal_address_id
                                        field="Locality"
                                    />
                                </div>
                                <TextInputWithValidation
                                    label="Region"
                                    name="form[region]"
                                    data_testid="input-region"
                                    value=postal_address_editor.region
                                    set_value=postal_address_editor.set_region
                                    optional=true
                                />
                                <EnumSelectWithValidation
                                    label="Country"
                                    name="form[country]"
                                    data_testid="select-country"
                                    value=postal_address_editor.country
                                    set_value=postal_address_editor.set_country
                                    validation_result=postal_address_editor.validation_result
                                    object_id=postal_address_editor.postal_address_id
                                    field="Country"
                                />
                                <div class="card-actions justify-end mt-4">
                                    <button
                                        type="submit"
                                        name="form[intent]"
                                        value=move || if is_new { "create" } else { "update" }
                                        data-testid="btn-save"
                                        class="btn btn-primary"
                                        prop:disabled=move || is_disabled() || !is_valid_addr()
                                    >
                                        "Save"
                                    </button>

                                    <button
                                        type="submit"
                                        name="form[intent]"
                                        value="copy_as_new"
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
                                        name="form[intent]"
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
