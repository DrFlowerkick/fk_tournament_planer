//! Postal Address Edit Module

use app_core::PostalAddress;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::postal_address::save_postal_address_inner;
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, TextInput},
    enum_utils::EditAction,
    hooks::{
        use_on_cancel::use_on_cancel,
        use_scroll_into_view::use_scroll_h2_into_view,
        use_url_navigation::{
            MatchedRouteHandler, UseMatchedRouteNavigationReturn, use_matched_route_navigation,
        },
    },
    params::{AddressIdQuery, EditActionParams, FilterNameQuery, ParamQuery},
    server_fn::postal_address::SavePostalAddress,
    state::{
        EditorContextWithResource, object_table::ObjectEditorMapContext,
        postal_address::PostalAddressEditorContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

#[component]
pub fn EditPostalAddress() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseMatchedRouteNavigationReturn {
        url_is_matched_route,
        ..
    } = use_matched_route_navigation();

    let edit_action = EditActionParams::use_param_query();
    let address_id = AddressIdQuery::use_param_query();

    // --- local state ---
    let postal_address_editor_map =
        expect_context::<ObjectEditorMapContext<PostalAddressEditorContext, AddressIdQuery>>();

    let editor = Signal::derive(move || {
        if let Some(id) = address_id.get()
            && let Some(editor) = postal_address_editor_map.get_editor(id)
            && match edit_action.get() {
                Some(EditAction::Edit) => {
                    editor.id.get().is_some() && editor.version.get().is_some()
                }
                Some(EditAction::New) | Some(EditAction::Copy) => {
                    editor.id.get().is_some() && editor.version.get().is_none()
                }
                None => false,
            }
        {
            Some(editor)
        } else {
            None
        }
    });

    // remove unsaved editor (no origin) on unmount
    on_cleanup(move || {
        if let Some(id) = address_id.get_untracked()
            && let Some(editor) = postal_address_editor_map.get_editor_untracked(id)
            && editor.id.get_untracked().is_some()
            && editor.version.get_untracked().is_none()
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
                            data-testid="action-btn-close-edit-form"
                        >
                            <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                        </button>
                    </div>
                    {move || {
                        editor
                            .try_get()
                            .flatten()
                            .map(|ed| {
                                view! { <PostalAddressForm postal_address_editor=ed /> }.into_any()
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                        <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                        <p class="text-2xl font-bold text-center">
                                            {move || match edit_action.try_get().flatten() {
                                                Some(EditAction::New) => {
                                                    "Press 'New Postal Address' to create a new postal address."
                                                }
                                                Some(EditAction::Edit) => {
                                                    "Please select a postal address from the list."
                                                }
                                                Some(EditAction::Copy) => {
                                                    "Press 'Copy selected Postal Address' to create a new postal address based upon the selected one."
                                                }
                                                None => "",
                                            }}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            })
                    }}
                </div>
            </div>
        </Show>
    }
}

#[component]
fn PostalAddressForm(postal_address_editor: PostalAddressEditorContext) -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseMatchedRouteNavigationReturn {
        url_matched_route_update_queries,
        ..
    } = use_matched_route_navigation();
    let navigate = use_navigate();

    let edit_action = EditActionParams::use_param_query();

    let post_save_callback = Callback::new(move |pa: PostalAddress| {
        if let Some(edit_action) = edit_action.get()
            && matches!(edit_action, EditAction::New | EditAction::Copy)
        {
            let pa_id = pa.get_id().to_string();
            let key_value = vec![
                (AddressIdQuery::KEY, pa_id.as_str()),
                (FilterNameQuery::KEY, pa.get_name()),
            ];
            // we need to use extend here, because the callback is executed in the route of
            // the list view
            let nav_url = url_matched_route_update_queries(
                key_value,
                MatchedRouteHandler::Extend(EditAction::Edit.to_string().as_str()),
            );
            navigate(
                &nav_url,
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });
    postal_address_editor
        .post_save_callback
        .set_value(Some(post_save_callback));

    let on_submit = move || {
        if let Some(pa) = postal_address_editor.local_read_only.get()
            && pa.validate().is_ok()
        {
            postal_address_editor.increment_optimistic_version();
            let data = SavePostalAddress { postal_address: pa };
            #[cfg(feature = "test-mock")]
            {
                let save_action = Action::new(|pa: &SavePostalAddress| {
                    let pa = pa.clone();
                    async move {
                        let result = save_postal_address_inner(pa.postal_address).await;
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
                postal_address_editor.save_postal_address.dispatch(data);
            }
        }
    };

    view! {
        // --- Address Form ---
        <div data-testid="form-address">
            <form on:submit:capture=move |ev| {
                ev.prevent_default();
                on_submit();
            }>
                // --- Address Form Fields ---
                <fieldset class="space-y-4 contents">
                    // Hidden meta fields the server expects (id / version)
                    <input
                        type="hidden"
                        data-testid="hidden-id"
                        prop:value=move || {
                            postal_address_editor.id.get().unwrap_or(Uuid::nil()).to_string()
                        }
                    />
                    <input
                        type="hidden"
                        data-testid="hidden-version"
                        prop:value=move || {
                            postal_address_editor.version.get().unwrap_or_default()
                        }
                    />
                    <TextInput
                        label="Name"
                        data_testid="input-name"
                        value=postal_address_editor.name
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_name)
                        validation_result=postal_address_editor.validation_result
                        object_id=postal_address_editor.id
                        field="name"
                    />
                    <TextInput
                        label="Street & number"
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
                        data_testid="input-region"
                        value=postal_address_editor.region
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_region)
                        optional=true
                    />
                    <EnumSelect
                        label="Country"
                        data_testid="select-country"
                        value=postal_address_editor.country
                        action=InputCommitAction::WriteAndSubmit(postal_address_editor.set_country)
                        validation_result=postal_address_editor.validation_result
                        object_id=postal_address_editor.id
                        field="country"
                    />
                </fieldset>
            </form>
        </div>
    }
}
