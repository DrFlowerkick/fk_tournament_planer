//! provides fn call for on cancel actions in error banners

use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_matched, use_navigate, use_url},
};

// ToDo: use use_query_navigation
pub fn use_on_cancel() -> Callback<()> {
    let navigate = use_navigate();
    let matched_route = use_matched();
    let url = use_url();

    let on_cancel = Callback::new(move |()| {
        let mut path = matched_route.get();
        if path != "/" {
            // cut off last segment
            if let Some(pos) = path.rfind('/') {
                path.truncate(pos);
            }
            if path.is_empty() {
                path = "/".to_string();
            }
        }
        let query_string = url.get().search_params().to_query_string();
        let final_path = format!("{}{}", path, query_string);
        navigate(
            &final_path,
            NavigateOptions {
                scroll: false,
                ..Default::default()
            },
        );
    });
    on_cancel
}
