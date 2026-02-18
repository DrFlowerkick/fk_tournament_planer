//! Postal Address Edit Module

#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::{SavePostalAddressFormData, save_postal_address_inner};
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, TextInput},
    enum_utils::EditAction,
    error::strategy::handle_write_error,
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::{AddressIdQuery, EditActionParams, FilterNameQuery, ParamQuery},
    server_fn::postal_address::SavePostalAddress,
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        postal_address::PostalAddressEditorContext, toast_state::ToastContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

#[component]
pub fn EditPostalAddress() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_matched_route_update_query,
        url_matched_route_update_queries,
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let edit_action = EditActionParams::use_param_query();
    let intent = Signal::derive(move || {
        edit_action.get().map(|action| match action {
            EditAction::Edit => "update".to_string(),
            EditAction::New | EditAction::Copy => "create".to_string(),
        })
    });

    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // selected address id from url query, if any
    let address_id = AddressIdQuery::use_param_query();

    // --- local state ---
    let postal_address_editor = expect_context::<PostalAddressEditorContext>();

    // --- state initialization & effects ---
    // We have the following cases:
    // 1) postal_address is Some -> an address was loaded
    // 1a) if last segment of matched route is "edit", we are editing an existing address
    // 1b) if last segment of matched route is "copy", we are copying an existing address as new
    // 1c) if last segment of matched route is "new", we navigate to edit
    // 2) postal_address is None -> no address was loaded
    // 2a) if last segment of matched route is "new", we are creating a new address
    // 2b) if last segment of matched route is "edit", we assume that no item was selected in
    //     the list and we show a message to select an address from the list.
    // 2c) if last segment of matched route is "copy", we show the message to select an address
    // from the list, because copy only makes sense if an address is selected.
    let (show_form, set_show_form) = signal(false);
    Effect::new({
        let navigate = navigate.clone();
        move || {
            match edit_action.get() {
                Some(EditAction::Edit) => {
                    // show form, if an address is loaded
                    set_show_form
                        .set(postal_address_editor.has_origin() && address_id.get().is_some());
                }
                Some(EditAction::Copy) => {
                    if let Some(id) = address_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            AddressIdQuery::KEY,
                            id.to_string().as_str(),
                            MatchedRouteHandler::ReplaceSegment(
                                EditAction::Edit.to_string().as_str(),
                            ),
                        );
                        navigate(
                            &nav_url,
                            NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    } else if postal_address_editor.has_origin() {
                        // prepare copy in editor
                        postal_address_editor.prepare_copy();
                        set_show_form.set(true);
                    } else if postal_address_editor.id.with(|id| id.is_some()) && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if postal_address_editor.id.with(|id| id.is_some()) {
                        // No origin, id is present, form is not shown -> show form
                        set_show_form.set(true);
                    } else {
                        // if there is no id, it means that no address was loaded, so we show the message to select an address from the list.
                        set_show_form.set(false);
                    }
                }
                Some(EditAction::New) => {
                    if let Some(id) = address_id.get() {
                        // if the user selected a table entry, we navigate to edit with the selected id
                        let nav_url = url_matched_route_update_query(
                            AddressIdQuery::KEY,
                            id.to_string().as_str(),
                            MatchedRouteHandler::ReplaceSegment(
                                EditAction::Edit.to_string().as_str(),
                            ),
                        );
                        navigate(
                            &nav_url,
                            NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    } else if postal_address_editor.has_origin()
                        || postal_address_editor.id.with(|id| id.is_none())
                    {
                        // if there is an origin or no id is set, create new postal address in editor and show form
                        postal_address_editor.new_postal_address();
                        set_show_form.set(true);
                    } else if postal_address_editor.id.with(|id| id.is_some()) && show_form.get() {
                        // No origin, id is present, form is shown -> everything is set
                    } else if postal_address_editor.id.with(|id| id.is_some()) {
                        // No origin, id is present, form is not shown -> show form
                        set_show_form.set(true);
                    } else {
                        // if there is no id, it means that no address was loaded, so we show the message to select an address from the list.
                        set_show_form.set(false);
                    }
                }
                None => set_show_form.set(false),
            }
        }
    });

    // cancel function for cancel button
    let on_cancel = use_on_cancel();

    // --- Server Actions ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();
    let save_postal_address_pending = save_postal_address.pending();
    activity_tracker.track_pending_memo(component_id.get_value(), save_postal_address_pending);

    // ToDo: with auto save and parallel editing, refetch is done automatically. Delete this dummy refetch.
    let refetch = Callback::new(move |_| {});

    // handle save result
    Effect::new(move || {
        if let Some(spa_result) = save_postal_address.value().get()
            && let Some(edit_action) = edit_action.get()
        {
            save_postal_address.clear();
            match spa_result {
                Ok(pa) => {
                    match edit_action {
                        EditAction::New | EditAction::Copy => {
                            let pa_id = pa.get_id().to_string();
                            let key_value = vec![
                                (AddressIdQuery::KEY, pa_id.as_str()),
                                (FilterNameQuery::KEY, pa.get_name()),
                            ];
                            let nav_url = url_matched_route_update_queries(
                                key_value,
                                MatchedRouteHandler::ReplaceSegment(
                                    EditAction::Edit.to_string().as_str(),
                                ),
                            );
                            navigate(
                                &nav_url,
                                NavigateOptions {
                                    replace: true,
                                    scroll: false,
                                    ..Default::default()
                                },
                            );
                        }
                        EditAction::Edit => {
                            if !postal_address_editor.check_optimistic_version(pa.get_version()) {
                                // version mismatch, likely due to parallel editing
                                // this should not happen, because version mismatch should be caught
                                // by the server and returned as error, but we handle it here just in case
                                leptos::logging::log!(
                                    "Version mismatch after saving Postal Address. Expected version: {:?}, actual version: {:?}. This might be caused by parallel editing.",
                                    postal_address_editor.version.get(),
                                    pa.get_version()
                                );
                            }
                        }
                    }
                    postal_address_editor.set_postal_address(pa);
                }
                Err(err) => {
                    leptos::logging::log!("Error saving Postal Address: {:?}", err);
                    // version reset for parallel editing
                    postal_address_editor.reset_version_to_origin();
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
    });

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Show when=move || edit_action.get().is_some() fallback=|| "Page not found.".into_view()>
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    <div class="flex justify-between items-center">
                        <h2 class="card-title" node_ref=scroll_ref>
                            {move || match edit_action.get() {
                                Some(EditAction::New) => "New Postal Address",
                                Some(EditAction::Edit) => "Edit Postal Address",
                                Some(EditAction::Copy) => "Copy Postal Address",
                                None => "",
                            }}
                        </h2>
                        <button
                            class="btn btn-square btn-ghost btn-sm"
                            on:click=move |_| on_cancel.run(())
                            aria-label="Close"
                            data-testid="action-btn-close"
                        >
                            <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                        </button>
                    </div>
                    <Show
                        when=move || show_form.get()
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
                                        if postal_address_editor
                                            .validation_result
                                            .with(|vr| vr.is_err())
                                        {
                                            return;
                                        }
                                        let data = SavePostalAddress {
                                            form: SavePostalAddressFormData {
                                                id: postal_address_editor.id.get().unwrap_or(Uuid::nil()),
                                                version: postal_address_editor
                                                    .version
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
                                                intent: intent.get(),
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
                                        if postal_address_editor
                                            .validation_result
                                            .with(|vr| vr.is_err())
                                        {
                                            ev.prevent_default();
                                        } else {
                                            postal_address_editor.increment_version();
                                        }
                                    }
                                }
                            >
                                // --- Address Form Fields ---
                                <fieldset class="space-y-4 contents">
                                    // Hidden meta fields the server expects (id / version)
                                    <div class="flex flex-col gap-2">
                                        <input
                                            type="text"
                                            class="text-primary"
                                            name="form[id]"
                                            data-testid="hidden-id"
                                            prop:value=move || {
                                                postal_address_editor
                                                    .id
                                                    .get()
                                                    .unwrap_or(Uuid::nil())
                                                    .to_string()
                                            }
                                        />
                                        <input
                                            type="text"
                                            class="text-secondary"
                                            name="form[version]"
                                            data-testid="hidden-version"
                                            readonly
                                            prop:value=move || {
                                                postal_address_editor.version.get().unwrap_or_default()
                                            }
                                        />
                                    </div>
                                    <input
                                        type="hidden"
                                        name="form[intent]"
                                        data-testid="intent"
                                        prop:value=move || intent.get()
                                    />
                                    <TextInput
                                        label="Name"
                                        name="form[name]"
                                        data_testid="input-name"
                                        value=postal_address_editor.name
                                        action=InputCommitAction::WriteAndSubmit(
                                            postal_address_editor.set_name,
                                        )
                                        validation_result=postal_address_editor.validation_result
                                        object_id=postal_address_editor.id
                                        field="Name"
                                    />
                                    <TextInput
                                        label="Street & number"
                                        name="form[street]"
                                        data_testid="input-street"
                                        value=postal_address_editor.street
                                        action=InputCommitAction::WriteAndSubmit(
                                            postal_address_editor.set_street,
                                        )
                                        validation_result=postal_address_editor.validation_result
                                        object_id=postal_address_editor.id
                                        field="Street"
                                    />
                                    <div class="grid grid-cols-2 gap-4">
                                        <TextInput
                                            label="Postal code"
                                            name="form[postal_code]"
                                            data_testid="input-postal_code"
                                            value=postal_address_editor.postal_code
                                            action=InputCommitAction::WriteAndSubmit(
                                                postal_address_editor.set_postal_code,
                                            )
                                            validation_result=postal_address_editor.validation_result
                                            object_id=postal_address_editor.id
                                            field="PostalCode"
                                        />
                                        <TextInput
                                            label="City"
                                            name="form[locality]"
                                            data_testid="input-locality"
                                            value=postal_address_editor.locality
                                            action=InputCommitAction::WriteAndSubmit(
                                                postal_address_editor.set_locality,
                                            )
                                            validation_result=postal_address_editor.validation_result
                                            object_id=postal_address_editor.id
                                            field="Locality"
                                        />
                                    </div>
                                    <TextInput
                                        label="Region"
                                        name="form[region]"
                                        data_testid="input-region"
                                        value=postal_address_editor.region
                                        action=InputCommitAction::WriteAndSubmit(
                                            postal_address_editor.set_region,
                                        )
                                        optional=true
                                    />
                                    <EnumSelect
                                        label="Country"
                                        name="form[country]"
                                        data_testid="select-country"
                                        value=postal_address_editor.country
                                        action=InputCommitAction::WriteAndSubmit(
                                            postal_address_editor.set_country,
                                        )
                                        validation_result=postal_address_editor.validation_result
                                        object_id=postal_address_editor.id
                                        field="Country"
                                    />
                                </fieldset>
                            </ActionForm>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
