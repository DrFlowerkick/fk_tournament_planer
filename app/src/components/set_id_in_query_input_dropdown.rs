//! General input dropdown component to set an ID in the query string.

use crate::hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation};
use app_core::utils::id_version::VersionId;
use leptos::{prelude::*, task::spawn_local, web_sys};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

use leptos::logging::log;

#[derive(Clone)]
pub struct SetIdInQueryInputDropdownProperties<I, RenderFn>
where
    I: VersionId + Clone + Send + Sync + 'static,
    RenderFn: Fn(&I) -> AnyView + Clone + Send + Sync + 'static,
{
    pub key: &'static str,
    pub placeholder: &'static str,
    pub name: RwSignal<String>,
    pub search_text: RwSignal<String>,
    pub list_items: Signal<Vec<I>>,
    pub render_item: RenderFn,
}

#[component]
pub fn SetIdInQueryInputDropdown<I>(
    props: SetIdInQueryInputDropdownProperties<
        I,
        impl Fn(&I) -> AnyView + Clone + Send + Sync + 'static,
    >,
) -> impl IntoView
where
    I: VersionId + Clone + Send + Sync + 'static,
{
    // ---- get properties ----
    let SetIdInQueryInputDropdownProperties {
        key,
        placeholder,
        name,
        search_text,
        list_items,
        render_item,
    } = props;
    // ---- initialize query navigation ----
    let UseQueryNavigationReturn {
        get,
        update,
        nav_url,
        ..
    } = use_query_navigation();

    // ---- search_text, dropdown-status & keyboard-highlight ----
    let (open, set_open) = signal(false);
    let (hi, set_hi) = signal::<Option<usize>>(None);

    // selection handler
    let select_idx = move |i: usize| {
        if let Some(item) = list_items.read_untracked().get(i) {
            // 1) update UI state
            search_text.set("".to_string());
            set_open.set(false);
            set_hi.set(None);
            // 2) check if id has changed
            let item_id = item.get_id_version().get_id();
            let current_id = get(key).and_then(|v| Uuid::parse_str(&v).ok());
            if item_id == current_id {
                // no change -> just reset name
                name.notify();
                return;
            }
            // 3) update URL query parameter with new id
            update(
                key,
                &item
                    .get_id_version()
                    .get_id()
                    .unwrap_or_default()
                    .to_string(),
            );
            let navigate = use_navigate();
            navigate(
                &nav_url.get(),
                NavigateOptions {
                    // replace=true prevents „history spam“
                    replace: true,
                    ..Default::default()
                },
            );
        }
    };

    // keyboard control
    let on_key = move |ev: web_sys::KeyboardEvent| {
        let len = list_items.read_untracked().len();
        match ev.key().as_str() {
            "ArrowDown" if len > 0 => {
                ev.prevent_default();
                let next = hi.get().map(|i| (i + 1) % len).unwrap_or(0);
                set_hi.set(Some(next));
                set_open.set(true);
            }
            "ArrowUp" if len > 0 => {
                ev.prevent_default();
                let next = hi.get().map(|i| (i + len - 1) % len).unwrap_or(len - 1);
                set_hi.set(Some(next));
                set_open.set(true);
            }
            "Enter" => {
                if let Some(i) = hi.get() {
                    ev.prevent_default();
                    log!("Selecting index via Enter: {}", i);
                    select_idx(i);
                }
            }
            "Escape" => {
                search_text.set("".to_string());
                set_open.set(false);
                set_hi.set(None);
                // reset name to last selected item
                name.notify();
            }
            _ => {}
        }
    };

    // handle blur
    let on_blur = move |_| {
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            search_text.set("".to_string());
            set_open.set(false);
            set_hi.set(None);
            // reset name to last selected item
            name.notify();
        });
    };

    view! {
        <div class=move || {
            format!("dropdown w-full {}", if open.get() { "dropdown-open" } else { "" })
        }>
            <input
                type="text"
                class="input input-bordered w-full"
                prop:value=move || name.get()
                data-testid=format!("{}-search-input", key)
                placeholder=placeholder
                on:input=move |ev| {
                    search_text.set(event_target_value(&ev));
                    set_open.set(true);
                    set_hi.set(None);
                }
                on:focus=move |_| {
                    if search_text.get().is_empty() {
                        search_text.set(name.get());
                    }
                    set_open.set(true);
                }
                on:keydown=on_key
                on:blur=on_blur
                autocomplete="off"
                role="combobox"
                aria-expanded=move || {
                    if open.get() && !list_items.get().is_empty() { "true" } else { "false" }
                }
                aria-controls=format!("{}-suggest", key)
            />

            {move || {
                let render_item = render_item.clone();
                open.get()
                    .then(|| {
                        view! {
                            <ul
                                id=format!("{}-suggest", key)
                                data-testid=format!("{}-search-suggest", key)
                                aria-busy=move || {
                                    if list_items.get().is_empty() { "true" } else { "false" }
                                }
                                class="dropdown-content menu menu-sm bg-base-100 rounded-box z-[1] mt-1 w-full p-0 shadow max-h-72 overflow-auto"
                                role="listbox"
                            >
                                {move || {
                                    let render_item = render_item.clone();
                                    if list_items.get().is_empty() {
                                        view! {
                                            <li class="px-3 py-2 text-sm text-base-content/70">
                                                "Searching…"
                                            </li>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <For
                                                each=move || {
                                                    list_items.get().clone().into_iter().enumerate()
                                                }
                                                key=|(_i, a)| a.get_id_version()
                                                children=move |(i, a)| {
                                                    let is_hi = move || {
                                                        hi.get().map(|j| j == i).unwrap_or(false)
                                                    };
                                                    let opt_id = format!("{}-option-{}", key, i);

                                                    view! {
                                                        <li
                                                            id=opt_id.clone()
                                                            data-testid=format!("{}-search-suggest-item", key)
                                                            role="option"
                                                            aria-selected=move || if is_hi() { "true" } else { "false" }
                                                            class:active=move || is_hi()
                                                        >
                                                            <p
                                                                class="flex flex-col items-start gap-0.5"
                                                                class:active=move || is_hi()
                                                                class:bg-base-200=move || is_hi()
                                                                on:mouseenter=move |_| set_hi.set(Some(i))
                                                                on:mousedown=move |_| select_idx(i)
                                                            >
                                                                {render_item(&a)}
                                                            </p>
                                                        </li>
                                                    }
                                                }
                                            />
                                        }
                                            .into_any()
                                    }
                                }}
                            </ul>
                        }
                            .into_any()
                    })
            }}
        </div>
    }
}
