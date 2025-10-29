// search for postal address by name

use super::{AddressParams, server_fn::load_postal_address_dummy};
use crate::{AppError, banner::AcknowledgmentAndNavigateBanner};
use app_core::{CrTopic, PostalAddress};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
};
use uuid::Uuid;

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();

    // load existing address when `id` is Some(...)
    let addr_res: Resource<Result<PostalAddress, AppError>> = Resource::new(
        move || params.get(),
        move |maybe_id| async move {
            let navigate = use_navigate();
            match maybe_id {
                // AppResult<PostalAddress>
                Ok(AddressParams { uuid: Some(id) }) => match load_postal_address_dummy(id).await {
                    Ok(Some(pa)) => Ok(pa),
                    Ok(None) => {
                        navigate(
                            "/postal-address",
                            NavigateOptions {
                                replace: true,
                                ..Default::default()
                            },
                        );
                        Ok(Default::default())
                    }
                    Err(e) => Err(e),
                },
                // new form or bad uuid: no loading delay
                _ => Ok(Default::default()),
            }
        },
    );

    // signals for address fields
    let (name, set_name) = signal(String::new());
    let (id, set_id) = signal(None::<Uuid>);
    let (version, set_version) = signal(0_u32);
    let (topic, set_topic) = signal(None::<CrTopic>);

    /*let socket_handler = move |msg: &CrSocketMsg| {
        match msg.0 {
            CrMsg::AddressUpdated { version: meta_version, .. } => {
                if meta_version > version.get_untracked() {
                    addr_res.refetch();
                }
            }
        }
    };*/
    //use_client_registry_topic(topic, socket_handler);

    view! {
        <Transition fallback=move || {
            view! {
                <div>
                    <p>"Searching for address..."</p>
                </div>
            }
        }>
            {move || {
                addr_res
                    .get()
                    .map(|res| match res {
                        Err(msg) => {
                            // --- General Load Error Banner ---
                            view! {
                                <AcknowledgmentAndNavigateBanner
                                    msg=format!("An unexpected error occurred during load: {msg}")
                                    ack_btn_text="Reload"
                                    ack_action=move || addr_res.refetch()
                                    nav_btn_text="Reset"
                                    navigate_url="/postal-address".into()
                                />
                            }
                                .into_any()
                        }
                        Ok(addr) => {
                            set_name.set(addr.get_name().to_string());
                            set_id.set(addr.get_id());
                            set_version.set(addr.get_version().unwrap_or_default());
                            if let Some(id) = addr.get_id() {
                                let new_topic = CrTopic::Address(id);
                                set_topic.set(Some(new_topic));
                            }
                            ().into_any()
                        }
                    })
            }} <p>"Postal Address Search (debug version)"</p> <p>{name}</p>
            <p>{move || format!("{}", id.get().unwrap_or_default().to_string())}</p>
            <p>{version}</p>
            <p>{move || format!("{}", topic.get().map(|t| t.to_string()).unwrap_or_default())}</p>
        </Transition>
    }
}
