//! Sport Configuration Search Component

use app_core::{CrTopic, SportConfig};
use app_utils::{
    components::{
        banner::AcknowledgmentAndNavigateBanner,
        set_id_in_query_input_dropdown::{
            SetIdInQueryInputDropdown, SetIdInQueryInputDropdownProperties,
        },
    },
    error::AppError,
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{SportConfigIdQuery, use_sport_id_query},
    server_fn::sport_config::{list_sport_configs, load_sport_config},
    state::global_state::{GlobalState, GlobalStateStoreFields},
};
use cr_leptos_axum_socket::use_client_registry_socket;
//use cr_single_instance::use_client_registry_sse;
use leptos::{logging::log, prelude::*};
use leptos_router::{components::A, hooks::use_query};
use reactive_stores::Store;
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn SearchSportConfig() -> impl IntoView {
    // get id's from query params
    let sport_id = use_sport_id_query();
    let sport_config_query = use_query::<SportConfigIdQuery>();
    let UseQueryNavigationReturn {
        url_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();

    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    let sport_plugin = move || {
        if let Some(sport_id) = sport_id.get() {
            sport_plugin_manager.get().get_web_ui(&sport_id)
        } else {
            log!("No valid sport_id in query params. Searching sport config is disabled.");
            None
        }
    };

    let sport_name = move || {
        if let Some(plugin) = sport_plugin() {
            plugin.name()
        } else {
            "Unknown Sport"
        }
    };

    // signals for dropdown
    let name = RwSignal::new(String::new());
    let search_text = RwSignal::new(String::new());

    // signals for client registry
    let (id, set_id) = signal(None::<Uuid>);
    let (topic, set_topic) = signal(None::<CrTopic>);
    let (version, set_version) = signal(0_u32);

    // load existing sport config when query contains sport_config_id
    let sport_config_res: Resource<Result<SportConfig, AppError>> = Resource::new(
        move || sport_config_query.get(),
        move |maybe_id| async move {
            match maybe_id {
                Ok(SportConfigIdQuery {
                    sport_config_id: Some(id),
                }) => match load_sport_config(id).await {
                    Ok(Some(sc)) => Ok(sc),
                    Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                    Err(e) => Err(e),
                },
                Ok(SportConfigIdQuery {
                    sport_config_id: None,
                }) => {
                    // no sport config id: no loading delay
                    Ok(Default::default())
                }
                Err(e) => Err(AppError::Other(e.to_string())),
            }
        },
    );

    let is_sport_config_res_error = move || matches!(sport_config_res.get(), Some(Err(_)));

    let refetch = Arc::new(move || sport_config_res.refetch());
    // update sport config via socket
    use_client_registry_socket(topic, version, refetch);
    // update sport config via sse
    //use_client_registry_sse(topic, version, refetch);

    // load possible sport configs from search_text
    let sport_config_list = Resource::new(
        move || (sport_id.get(), search_text.get()),
        |(maybe_sport_id, name)| async move {
            if name.len() > 0
                && let Some(sport_id) = maybe_sport_id
            {
                return list_sport_configs(sport_id, name).await;
            }
            Ok(vec![])
        },
    );

    let is_sport_config_list_error = move || matches!(sport_config_list.get(), Some(Err(_)));

    let is_disabled = move || {
        sport_config_res.get().is_none()
            || is_sport_config_res_error()
            || is_sport_config_list_error()
    };

    // list of postal addresses matching search_text
    let results = Signal::derive(move || {
        sport_config_list
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });

    // reset url when unexpectedly no sport config found
    let reset_url =
        move || url_matched_route_remove_query("sport_config_id", MatchedRouteHandler::Keep);

    let props = SetIdInQueryInputDropdownProperties {
        key: "sport_config_id",
        name,
        placeholder: "Enter name of sport configuration you are searching...",
        search_text,
        list_items: results,
        render_item: move |c| {
            if let Some(sp) = sport_plugin() {
                sp.render_dropdown(&c)
            } else {
                view! { <span class="font-medium">{c.get_name()}</span> }.into_any()
            }
        },
    };

    {
        move || {
            let props = props.clone();
            sport_plugin()
                .map(|sp| {
                    view! {
                        <div
                            class="card w-full bg-base-100 shadow-xl"
                            data-testid="search-sport-config"
                        >
                            <div class="card-body">
                                <h2 class="card-title">
                                    {format!("Search {} Configuration", sport_name())}
                                </h2>
                                <Transition fallback=move || {
                                    view! {
                                        <div class="flex justify-center items-center p-4">
                                            <span class="loading loading-spinner loading-lg"></span>
                                        </div>
                                    }
                                }>
                                    {move || {
                                        sport_config_res
                                            .get()
                                            .map(|res| match res {
                                                Err(msg) => {
                                                    // --- General Load Error Banner ---
                                                    view! {
                                                        <AcknowledgmentAndNavigateBanner
                                                            msg=format!(
                                                                "An unexpected error occurred during load: {msg}",
                                                            )
                                                            ack_btn_text="Reload"
                                                            ack_action=move || sport_config_res.refetch()
                                                            nav_btn_text="Reset"
                                                            navigate_url=reset_url()
                                                        />
                                                    }
                                                        .into_any()
                                                }
                                                Ok(sport_config) => {
                                                    name.set(sport_config.get_name().to_string());
                                                    set_version
                                                        .set(sport_config.get_version().unwrap_or_default());
                                                    if sport_config.get_version().is_some() {
                                                        let new_topic = CrTopic::SportConfig(sport_config.get_id());
                                                        set_topic.set(Some(new_topic));
                                                        set_id.set(Some(sport_config.get_id()));
                                                    } else {
                                                        set_id.set(None);
                                                    }
                                                    ().into_any()
                                                }
                                            })
                                    }} <SetIdInQueryInputDropdown props=props />
                                    {move || {
                                        if let Some(Ok(sport_config)) = sport_config_res.get() {
                                            if sport_config.get_version().is_some() {
                                                sp.render_preview(&sport_config)
                                            } else {
                                                view! {
                                                    <div class="mt-4">
                                                        <p>"No sport configuration selected."</p>
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                        } else {
                                            ().into_any()
                                        }
                                    }} <div class="card-actions justify-end mt-4">
                                        <A
                                            href=move || url_matched_route_remove_query(
                                                "sport_config_id",
                                                MatchedRouteHandler::Extend("new_sc"),
                                            )
                                            attr:class="btn btn-primary"
                                            attr:data-testid="btn-new-sport-config"
                                            attr:disabled=is_disabled
                                        >
                                            "New"
                                        </A>
                                        <A
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("edit_sc"),
                                            )
                                            attr:class="btn btn-secondary"
                                            attr:data-testid="btn-edit-sport-config"
                                            attr:disabled=move || is_disabled() || id.get().is_none()
                                        >
                                            "Edit"
                                        </A>
                                    </div>
                                </Transition>
                            </div>
                        </div>
                    }
                })
        }
    }
}
