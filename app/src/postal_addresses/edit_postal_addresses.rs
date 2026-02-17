//! Postal Address Edit Module

use app_core::{CrTopic, PostalAddress, utils::id_version::IdVersion};
#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::{SavePostalAddressFormData, save_postal_address_inner};
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, TextInput},
    enum_utils::EditAction,
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
    params::{AddressIdQuery, EditActionParams, FilterNameQuery, ParamQuery},
    server_fn::postal_address::{SavePostalAddress, load_postal_address},
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        postal_address::PostalAddressEditorContext, toast_state::ToastContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
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

    // parallel editing
    let (topic, set_topic) = signal(None::<CrTopic>);
    let (version, set_version) = signal(0_u32);
    use_client_registry_socket(topic.into(), version.into(), refetch);

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
                                    set_topic=set_topic
                                    version=version
                                    set_version=set_version
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
fn EditPostalAddress(
    #[prop(into)] postal_address: Signal<Option<PostalAddress>>,
    set_topic: WriteSignal<Option<CrTopic>>,
    version: ReadSignal<u32>,
    set_version: WriteSignal<u32>,
    refetch: Callback<()>,
) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_matched_route,
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

    let postal_address_editor = PostalAddressEditorContext::new();
    // We have the following cases:
    // 1) postal_address is Some -> an address was loaded
    // 1a) if last segment of matched route is "edit", we are editing an existing address
    // 1b) if last segment of matched route is "copy", we are copying an existing address as new
    // 1c) if last segment of matched route is "new", we navigate to edit
    // 2) postal_address is None -> no address was loaded
    // 2a) if last segment of matched route is "new", we are creating a new address
    // 2b) if last segment of matched route is "edit", we assume that no item was selected in
    //     the list and we show a message to select an address from the list.
    // 2c) if last segment of matched route is "copy", we we navigate to edit.
    let (show_form, set_show_form) = signal(false);
    Effect::new({
        let navigate = navigate.clone();
        move || {
            if let Some(mut pa) = postal_address.get() {
                match edit_action.get() {
                    Some(EditAction::Edit) => {
                        // set topic and version for parallel editing
                        set_topic.set(Some(CrTopic::Address(pa.get_id())));
                        set_version.set(pa.get_version().unwrap_or_default());
                        // set state
                        postal_address_editor.set_postal_address(pa);
                        set_show_form.set(true);
                    }
                    Some(EditAction::Copy) => {
                        pa.set_id_version(IdVersion::new(Uuid::new_v4(), None))
                            .set_name("");
                        postal_address_editor.set_postal_address(pa);
                        set_show_form.set(true);
                    }
                    Some(EditAction::New) => {
                        let nav_url =
                            url_matched_route(MatchedRouteHandler::ReplaceSegment("edit"));
                        navigate(&nav_url, NavigateOptions::default());
                        set_show_form.set(false);
                    }
                    None => set_show_form.set(false),
                }
            } else {
                match edit_action.get() {
                    Some(EditAction::New) => {
                        postal_address_editor.new_postal_address();
                        set_show_form.set(true);
                    }
                    Some(EditAction::Copy) => {
                        let nav_url =
                            url_matched_route(MatchedRouteHandler::ReplaceSegment("edit"));
                        navigate(&nav_url, NavigateOptions::default());
                        set_show_form.set(false);
                    }
                    Some(EditAction::Edit) | None => set_show_form.set(false),
                }
            }
        }
    });

    // cancel function for cancel button
    let on_cancel = use_on_cancel();

    // --- Server Actions ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();
    let save_postal_address_pending = save_postal_address.pending();
    activity_tracker.track_pending_memo(component_id.get_value(), save_postal_address_pending);

    // handle save result
    Effect::new(move || {
        if let Some(spa_result) = save_postal_address.value().get()
            && let Some(edit_action) = edit_action.get()
        {
            save_postal_address.clear();
            match spa_result {
                Ok(pa) => match edit_action {
                    EditAction::New | EditAction::Copy => {
                        let pa_id = pa.get_id().to_string();
                        let key_value = vec![
                            (AddressIdQuery::KEY, pa_id.as_str()),
                            (FilterNameQuery::KEY, pa.get_name()),
                        ];
                        let nav_url = url_matched_route_update_queries(
                            key_value,
                            MatchedRouteHandler::ReplaceSegment("edit"),
                        );
                        navigate(&nav_url, NavigateOptions::default());
                    }
                    EditAction::Edit => {
                        if let Some(current_version) = pa.get_version()
                            && current_version != version.get()
                        {
                            // version mismatch, likely due to parallel editing
                            // this should not happen, because version mismatch should be caught
                            // by the server and returned as error, but we handle it here just in case
                            leptos::logging::log!(
                                "Version mismatch after saving Postal Address. Expected version: {}, actual version: {}. This might be caused by parallel editing.",
                                version.get(),
                                current_version
                            );
                            refetch.run(());
                        }
                    }
                },
                Err(err) => {
                    leptos::logging::log!("Error saving Postal Address: {:?}", err);
                    // version reset for parallel editing
                    set_version.set(postal_address.with(|maybe_pa| {
                        maybe_pa
                            .as_ref()
                            .and_then(|pa| pa.get_version())
                            .unwrap_or_default()
                    }));
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
                                            set_version.update(|v| *v += 1);
                                        }
                                    }
                                }
                            >
                                // --- Address Form Fields ---
                                <fieldset class="space-y-4 contents">
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
                                        object_id=postal_address_editor.postal_address_id
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
                                        object_id=postal_address_editor.postal_address_id
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
                                            object_id=postal_address_editor.postal_address_id
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
                                            object_id=postal_address_editor.postal_address_id
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
                                        object_id=postal_address_editor.postal_address_id
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
