//! Sport Config Edit Module

use app_core::SportConfig;
#[cfg(feature = "test-mock")]
use app_utils::server_fn::sport_config::save_sport_config_inner;
use app_utils::{
    components::inputs::{InputCommitAction, TextInput},
    enum_utils::EditAction,
    hooks::{
        use_on_cancel::use_on_cancel,
        use_scroll_into_view::use_scroll_h2_into_view,
        use_url_navigation::{
            MatchedRouteHandler, UseMatchedRouteNavigationReturn, use_matched_route_navigation,
        },
    },
    params::{EditActionParams, FilterNameQuery, ParamQuery, SportConfigIdQuery, SportIdQuery},
    server_fn::sport_config::SaveSportConfig,
    state::{
        EditorContext, EditorContextWithResource,
        global_state::{GlobalState, GlobalStateStoreFields},
        object_table::ObjectEditorMapContext,
        sport_config::SportConfigEditorContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn EditSportConfiguration() -> impl IntoView {
    // --- Hooks, Navigation & local and global state ---
    let UseMatchedRouteNavigationReturn {
        url_is_matched_route,
        ..
    } = use_matched_route_navigation();
    let edit_action = EditActionParams::use_param_query();
    let sport_config_id = SportConfigIdQuery::use_param_query();

    // sport id and plugin manager
    let sport_id = SportIdQuery::use_param_query();
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .try_with(|maybe_sport_id| {
                maybe_sport_id
                    .as_ref()
                    .and_then(|s_id| sport_plugin_manager.get().get_web_ui(s_id))
            })
            .flatten()
    };
    let sport_name = move || {
        if let Some(plugin) = sport_plugin() {
            plugin.name()
        } else {
            "Unknown Sport"
        }
    };

    // --- local state ---
    let sport_config_editor_map =
        expect_context::<ObjectEditorMapContext<SportConfigEditorContext, SportConfigIdQuery>>();

    let editor = Signal::derive(move || {
        if let Some(id) = sport_config_id.get()
            && let Some(editor) = sport_config_editor_map.get_editor(id)
            && match edit_action.get() {
                Some(EditAction::Edit) => editor.origin_signal().with(|origin| origin.is_some()),
                Some(EditAction::New) => editor.origin_signal().with(|origin| origin.is_none()),
                Some(EditAction::Copy) => editor.origin_signal().with(|origin| origin.is_none()),
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
        if let Some(id) = sport_config_id.get_untracked()
            && let Some(editor) = sport_config_editor_map.get_editor_untracked(id)
            && editor.origin_signal().with(|origin| origin.is_none())
        {
            sport_config_editor_map.remove_editor(id);
        }
    });

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Show
            when=move || edit_action.try_get().flatten().is_some()
            fallback=|| "Page not found.".into_view()
        >
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    <div class="flex justify-between items-center">
                        <h2 class="card-title" node_ref=scroll_ref>
                            {move || match edit_action.try_get().flatten() {
                                Some(EditAction::New) => {
                                    "New Sport Configuration for ".to_string() + sport_name()
                                }
                                Some(EditAction::Edit) => {
                                    "Edit Sport Configuration for ".to_string() + sport_name()
                                }
                                Some(EditAction::Copy) => {
                                    "Copy Sport Configuration for ".to_string() + sport_name()
                                }
                                None => "".to_string(),
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
                                view! { <SportConfigForm sport_config_editor=ed /> }.into_any()
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                                        <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                                        <p class="text-2xl font-bold text-center">
                                            {move || match edit_action.try_get().flatten() {
                                                Some(EditAction::New) => {
                                                    "Press 'New Sport Configuration' to create a new sport configuration."
                                                }
                                                Some(EditAction::Edit) => {
                                                    "Please select a sport configuration from the list."
                                                }
                                                Some(EditAction::Copy) => {
                                                    "Press 'Copy selected Sport Configuration' to create a new sport configuration based upon the selected one."
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
fn SportConfigForm(sport_config_editor: SportConfigEditorContext) -> impl IntoView {
    // --- Hooks, Navigation & local and global state ---
    let UseMatchedRouteNavigationReturn {
        url_matched_route_update_queries,
        ..
    } = use_matched_route_navigation();
    let navigate = use_navigate();

    let edit_action = EditActionParams::use_param_query();

    // sport id and plugin manager
    let sport_id = SportIdQuery::use_param_query();
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .try_with(|maybe_sport_id| {
                maybe_sport_id
                    .as_ref()
                    .and_then(|s_id| sport_plugin_manager.get().get_web_ui(s_id))
            })
            .flatten()
    };

    // provide local context for web ui plug ins
    provide_context(sport_config_editor);

    let post_save_callback = Callback::new(move |sc: SportConfig| {
        if let Some(edit_action) = edit_action.get()
            && matches!(edit_action, EditAction::New | EditAction::Copy)
        {
            let sc_id = sc.get_id().to_string();
            let key_value = vec![
                (SportConfigIdQuery::KEY, sc_id.as_str()),
                (FilterNameQuery::KEY, sc.get_name()),
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
    sport_config_editor
        .post_save_callback
        .set_value(Some(post_save_callback));

    let on_submit = move || {
        if let Some(sc) = sport_config_editor.local_read_only.get()
            && let Some(sport_plugin) = sport_plugin()
            && sc.validate(sport_plugin).is_ok()
        {
            sport_config_editor.increment_optimistic_version();
            let data = SaveSportConfig { sport_config: sc };
            #[cfg(feature = "test-mock")]
            {
                let save_action = Action::new(|sc: &SaveSportConfig| {
                    let sc = sc.clone();
                    async move {
                        let result = save_sport_config_inner(sc.sport_config).await;
                        leptos::web_sys::console::log_1(
                            &format!("Result of save sport config: {:?}", result).into(),
                        );
                        result
                    }
                });
                save_action.dispatch(data);
            }
            #[cfg(not(feature = "test-mock"))]
            {
                sport_config_editor.save_sport_config.dispatch(data);
            }
        }
    };

    view! {
        // --- Sport Config Form ---
        <div data-testid="form-sport-config">
            <form on:submit:capture=move |ev| {
                ev.prevent_default();
                on_submit();
            }>
                // --- Sport Config Form Fields ---
                <fieldset class="space-y-4 contents">
                    // Hidden meta fields the server expects (id / version)
                    <input
                        type="hidden"
                        data-testid="hidden-id"
                        prop:value=move || {
                            sport_config_editor.id.get().unwrap_or(Uuid::nil()).to_string()
                        }
                    />
                    <input
                        type="hidden"
                        data-testid="hidden-version"
                        prop:value=move || { sport_config_editor.version.get().unwrap_or_default() }
                    />
                    <TextInput
                        label="Name"
                        data_testid="input-name"
                        value=sport_config_editor.name
                        action=InputCommitAction::WriteAndSubmit(sport_config_editor.set_name)
                        validation_result=sport_config_editor.validation_result
                        object_id=sport_config_editor.id
                        field="name"
                    />
                    // Sport specific configuration UI
                    {move || { sport_plugin().map(|plugin| plugin.render_configuration()) }}
                </fieldset>
            </form>
        </div>
    }
}
