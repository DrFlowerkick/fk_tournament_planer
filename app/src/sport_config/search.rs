use crate::sport_config::server_fn::list_sport_configs;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn SearchSportConfig(sport_id: Uuid) -> impl IntoView {
    let (search_text, set_search_text) = signal(String::new());

    let configs = Resource::new(
        move || (sport_id, search_text.get()),
        move |(id, filter)| async move {
            let filter = if filter.is_empty() {
                None
            } else {
                Some(filter)
            };
            list_sport_configs(id, filter, Some(50)).await
        },
    );

    view! {
        <div class="card w-full bg-base-100 shadow-xl mt-4">
            <div class="card-body">
                <h2 class="card-title">"Sport Configurations"</h2>
                <div class="form-control">
                    <input
                        type="text"
                        placeholder="Search configs..."
                        class="input input-bordered w-full"
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        data-testid="search-sport-config-input"
                    />
                </div>
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Transition fallback=move || {
                                view! {
                                    <tr>
                                        <td colspan="2">"Loading..."</td>
                                    </tr>
                                }
                            }>
                                {move || {
                                    configs
                                        .get()
                                        .map(|res| match res {
                                            Ok(list) => {
                                                if list.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="2">"No configurations found."</td>
                                                        </tr>
                                                    }
                                                        .into_any()
                                                } else {
                                                    list.into_iter()
                                                        .map(|config| {
                                                            view! {
                                                                <tr data-testid=format!("config-row-{}", config.name)>
                                                                    <td>{config.name.clone()}</td>
                                                                    <td>
                                                                        <button class="btn btn-sm btn-ghost">"Edit"</button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        })
                                                        .collect_view()
                                                        .into_any()
                                                }
                                            }
                                            Err(e) => {
                                                view! {
                                                    <tr>
                                                        <td colspan="2" class="text-error">
                                                            {format!("Error: {}", e)}
                                                        </td>
                                                    </tr>
                                                }
                                                    .into_any()
                                            }
                                        })
                                }}
                            </Transition>
                        </tbody>
                    </table>
                </div>
                <div class="card-actions justify-end">
                    <button class="btn btn-primary" data-testid="new-sport-config-btn">
                        "New Configuration"
                    </button>
                </div>
            </div>
        </div>
    }
}
