//! Postal Address Edit Module

#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::{SavePostalAddressFormData, save_postal_address_inner};
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, TextInput},
    enum_utils::EditAction,
    error::{map_db_unique_violation_to_field_error, strategy::handle_write_error},
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
        EditorContext, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table::ObjectEditorMapContext, postal_address::PostalAddressEditorContext,
        toast_state::ToastContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

#[component]
pub fn EditPostalAddress() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_is_matched_route,
        ..
    } = use_query_navigation();
    let edit_action = EditActionParams::use_param_query();
    let address_id = AddressIdQuery::use_param_query();

    // --- local state ---
    let postal_address_editor_map =
        expect_context::<ObjectEditorMapContext<PostalAddressEditorContext, AddressIdQuery>>();

    let show_form = Signal::derive(move || {
        if let Some(id) = address_id.get()
            && let Some(editor) = postal_address_editor_map.get_editor(id)
        {
            match edit_action.get() {
                Some(EditAction::Edit) => editor.get_origin().is_some(),
                Some(EditAction::New) => editor.get_origin().is_none(),
                Some(EditAction::Copy) => editor.get_origin().is_none(),
                None => false,
            }
        } else {
            false
        }
    });

    // remove unsaved editor (no origin) on unmount
    on_cleanup(move || {
        if let Some(id) = address_id.get_untracked()
            && let Some(editor) = postal_address_editor_map.get_editor_untracked(id)
            && editor.get_origin().is_none()
        {
            postal_address_editor_map.remove_editor(id);
        }
    });

    // cancel function for close / cancel button
    let on_cancel = use_on_cancel();

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
                        fallback=move || {
                            view! {
                                <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                    <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                    <p class="text-2xl font-bold text-center">
                                        {move || match edit_action.get() {
                                            Some(EditAction::New) => {
                                                "Press 'New Postal Address' to create a new postal address."
                                            }
                                            Some(EditAction::Edit) => {
                                                "Please select a postal address from the list."
                                            }
                                            Some(EditAction::Copy) => {
                                                "Press 'Copy' of a selected postal address to create a new postal address based upon the selected one."
                                            }
                                            None => "",
                                        }}
                                    </p>
                                </div>
                            }
                        }
                    >
                        {move || {
                            postal_address_editor_map
                                .get_editor(address_id.get().unwrap_or_default())
                                .map(|editor| {
                                    view! { <PostalAddressForm postal_address_editor=editor /> }
                                })
                        }}
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn PostalAddressForm(postal_address_editor: PostalAddressEditorContext) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        url_matched_route_update_queries,
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

    // --- global state ---
    let toast_ctx = expect_context::<ToastContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

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
                    postal_address_editor.set_object(pa.clone());
                    if matches!(edit_action, EditAction::New | EditAction::Copy) {
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
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    }
                }
                Err(err) => {
                    // version reset for parallel editing
                    postal_address_editor.reset_version_to_origin();
                    // transform unique violation error into Validation Error for name, if any
                    if let Some(object_id) = postal_address_editor.id.get()
                        && let Some(field_error) =
                            map_db_unique_violation_to_field_error(&err, object_id, "name")
                    {
                        postal_address_editor
                            .set_unique_violation_error
                            .set(Some(field_error));
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

    view! {
        // --- Address Form ---
        <div data-testid="form-address">
            <ActionForm
                action=save_postal_address
                on:submit:capture=move |ev| {
                    #[cfg(feature = "test-mock")]
                    {
                        ev.prevent_default();
                        if postal_address_editor.validation_result.with(|vr| vr.is_err()) {
                            return;
                        }
                        postal_address_editor.increment_optimistic_version();
                        let data = SavePostalAddress {
                            form: SavePostalAddressFormData {
                                id: postal_address_editor.id.get().unwrap_or(Uuid::nil()),
                                version: postal_address_editor.version.get().unwrap_or_default(),
                                name: postal_address_editor.name.get().unwrap_or_default(),
                                street: postal_address_editor.street.get().unwrap_or_default(),
                                postal_code: postal_address_editor
                                    .postal_code
                                    .get()
                                    .unwrap_or_default(),
                                locality: postal_address_editor.locality.get().unwrap_or_default(),
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
                                    &format!("Result of save postal address: {:?}", result).into(),
                                );
                                result
                            }
                        });
                        save_action.dispatch(data);
                    }
                    #[cfg(not(feature = "test-mock"))]
                    {
                        if postal_address_editor.validation_result.with(|vr| vr.is_err()) {
                            ev.prevent_default();
                        } else {
                            postal_address_editor.increment_optimistic_version();
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
                            postal_address_editor.id.get().unwrap_or(Uuid::nil()).to_string()
                        }
                    />
                    <input
                        type="hidden"
                        name="form[version]"
                        data-testid="hidden-version"
                        readonly
                        prop:value=move || {
                            postal_address_editor.version.get().unwrap_or_default()
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
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_name)
                        validation_result=postal_address_editor.validation_result
                        object_id=postal_address_editor.id
                        field="name"
                    />
                    <TextInput
                        label="Street & number"
                        name="form[street]"
                        data_testid="input-street"
                        value=postal_address_editor.street
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_street)
                        validation_result=postal_address_editor.validation_result
                        object_id=postal_address_editor.id
                        field="street"
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
                            field="postal_code"
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
                            field="locality"
                        />
                    </div>
                    <TextInput
                        label="Region"
                        name="form[region]"
                        data_testid="input-region"
                        value=postal_address_editor.region
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_region)
                        optional=true
                    />
                    <EnumSelect
                        label="Country"
                        name="form[country]"
                        data_testid="select-country"
                        value=postal_address_editor.country
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_country)
                        validation_result=postal_address_editor.validation_result
                        object_id=postal_address_editor.id
                        field="country"
                    />
                </fieldset>
            </ActionForm>
        </div>
    }
}
