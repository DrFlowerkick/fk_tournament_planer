use crate::common::{get_element_by_test_id, get_test_root, lock_test};
use app_core::utils::{id_version::IdVersion, traits::ObjectIdVersion};
use app_utils::components::set_id_in_query_input_dropdown::{
    SetIdInQueryInputDropdown, SetIdInQueryInputDropdownProperties,
};
use gloo_timers::future::sleep;
use leptos::{
    mount::mount_to,
    prelude::*,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit},
};
use leptos_router::{components::Router, hooks::use_query, params::Params};
use std::time::Duration;
use uuid::Uuid;
use wasm_bindgen_test::*;

#[derive(Clone)]
struct TestItem {
    id_version: IdVersion,
    name: String,
}

impl ObjectIdVersion for TestItem {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct WasmTestParams {
    pub wasm_test_item_id: Option<Uuid>,
}

#[component]
fn WasmTestWrapper(items: Vec<TestItem>) -> impl IntoView {
    // This component is defined inside the test function
    let key = "wasm_test_item_id";
    let placeholder = "Enter name of test item...";
    let test_id_query = use_query::<WasmTestParams>();
    let name = RwSignal::new("".to_string());
    let search_text = RwSignal::new("".to_string());

    let list_items = Signal::derive(move || {
        let search = search_text.read().to_lowercase();
        if search.is_empty() {
            items.clone()
        } else {
            items
                .clone()
                .into_iter()
                .filter(|item| item.name.to_lowercase().contains(&search))
                .collect()
        }
    });

    Effect::new(move || {
        if let Ok(params) = test_id_query.get()
            && let Some(item_id) = params.wasm_test_item_id
        {
            let item_name = list_items
                .get_untracked()
                .iter()
                .find(|item| item.get_id_version().get_id() == Some(item_id))
                .map(|item| item.name.clone())
                .unwrap_or_default();
            name.set(item_name);
        } else {
            name.set("".to_string());
        }
    });

    let props = SetIdInQueryInputDropdownProperties {
        key,
        placeholder,
        name,
        search_text,
        list_items,
        render_item: move |item: &TestItem| view! { <div>{item.name.clone()}</div> }.into_any(),
    };
    view! {
        <div>
            <h2>"Test SetIdInQueryInputDropdown Component"</h2>
            <p data-testid="name">{move || format!("Selected Name: {}", props.name.read())}</p>
            <p data-testid="query">
                {move || {
                    if let Ok(params) = test_id_query.get() {
                        format!("Query ID: {:?}", params.wasm_test_item_id)
                    } else {
                        "Query ID: None".to_string()
                    }
                }}
            </p>
        </div>
        <SetIdInQueryInputDropdown props=props />
    }
}

#[wasm_bindgen_test]
async fn test_set_id_in_query_input_dropdown() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let items: Vec<TestItem> = vec![
        TestItem {
            id_version: IdVersion::new(Uuid::new_v4(), Some(1)),
            name: "Item 1".to_string(),
        },
        TestItem {
            id_version: IdVersion::new(Uuid::new_v4(), Some(1)),
            name: "Item 2".to_string(),
        },
        TestItem {
            id_version: IdVersion::new(Uuid::new_v4(), Some(1)),
            name: "Item 3".to_string(),
        },
    ];

    let _mount_guard = mount_to(get_test_root(), {
        let items = items.clone();
        move || {
            view! {
                <Router>
                    <WasmTestWrapper items=items />
                </Router>
            }
        }
    });

    sleep(Duration::from_millis(10)).await;

    let input = get_element_by_test_id("wasm_test_item_id-search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of test item..."
    );
    // simulate input: cast to HtmlInputElement and set value
    let input_elem = input.dyn_into::<HtmlInputElement>().unwrap();

    for index in -1..=(items.len() as i32) {
        input_elem.set_value("Item");
        let event = Event::new("input").unwrap();
        input_elem.dispatch_event(&event).unwrap();
        sleep(Duration::from_millis(50)).await;
        let suggest = get_element_by_test_id("wasm_test_item_id-search-suggest");
        for num in 1..=items.len() {
            assert!(
                suggest
                    .text_content()
                    .unwrap()
                    .contains(&format!("Item {}", num))
            );
        }

        // simulate selection of first suggestion via keyboard events
        // 1. ArrowUp / ArrowDown Event
        let key = if index < 0 { "ArrowUp" } else { "ArrowDown" };
        let num_key_down = if index < 0 { index.abs() } else { index + 1 };
        let index = index.rem_euclid(items.len() as i32) as usize;
        let id = items[index].id_version.get_id().unwrap();
        let init_down = KeyboardEventInit::new();
        init_down.set_key(key);
        init_down.set_code(key);
        init_down.set_bubbles(true);
        init_down.set_cancelable(true);
        let event_down =
            KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_down).unwrap();
        for _ in 0..num_key_down {
            input_elem.dispatch_event(&event_down).unwrap();
        }

        sleep(Duration::from_millis(10)).await;

        // 2. Enter Event
        let init_enter = KeyboardEventInit::new();
        init_enter.set_key("Enter");
        init_enter.set_code("Enter");
        init_enter.set_bubbles(true);
        init_enter.set_cancelable(true);
        let event_enter =
            KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_enter).unwrap();
        input_elem.dispatch_event(&event_enter).unwrap();

        sleep(Duration::from_millis(10)).await;

        let url_id = document()
            .location()
            .unwrap()
            .href()
            .unwrap()
            .split("wasm_test_item_id=")
            .last()
            .unwrap()
            .to_string();
        assert_eq!(url_id, id.to_string());

        let name = get_element_by_test_id("name");
        assert!(
            name.text_content()
                .unwrap()
                .contains(&format!("Selected Name: Item {}", index + 1))
        );
        let query = get_element_by_test_id("query");
        assert!(
            query
                .text_content()
                .unwrap()
                .contains(&format!("Query ID: Some({})", id))
        );
    }
}
