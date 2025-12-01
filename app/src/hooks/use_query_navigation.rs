//! Provides a hook for query-based navigation.

use leptos::prelude::*;
use leptos_router::{hooks::use_url, location::Url};

/// A hook that provides query-based navigation capabilities.
pub fn use_query_navigation() -> UseQueryNavigationReturn<
    impl Fn(&str, &str) + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) -> String + Clone + Copy + Send + Sync + 'static,
> {
    let url = use_url();
    let mut_url = RwSignal::new(Url::default());
    Effect::new(move || mut_url.set(url.get()));
    let update = move |key: &str, value: &str| {
        mut_url.update(|url| {
            url.search_params_mut()
                .replace(key.to_string(), value.to_string())
        });
    };
    let remove = move |key: &str| {
        mut_url.update(|url| {
            let _ = url.search_params_mut().remove(key);
        });
    };
    let path = Signal::derive(move || mut_url.get().path().to_string());
    let query_string = Signal::derive(move || mut_url.get().search_params().to_query_string());
    let nav_url = Signal::derive(move || format!("{}{}", path.get(), query_string.get()));
    let relative_sub_url = move |sub_path: &str| format!("{}{}", sub_path, query_string.get());
    UseQueryNavigationReturn {
        update,
        remove,
        path,
        query_string,
        nav_url,
        relative_sub_url,
    }
}

/// Return type of `use_query_navigation`.
pub struct UseQueryNavigationReturn<UpdateFn, RemoveFn, RelativeSubUrlFn>
where
    UpdateFn: Fn(&str, &str),
    RemoveFn: Fn(&str),
    RelativeSubUrlFn: Fn(&str) -> String,
{
    /// Function to update a query in the query map.
    pub update: UpdateFn,

    /// Function to remove a key from the query map.
    pub remove: RemoveFn,

    /// Function which returns the current query string.
    /// Use this in combination with <A> component for relative sub-urls.
    pub relative_sub_url: RelativeSubUrlFn,

    /// Signal to return current path.
    pub path: Signal<String>,

    /// Signal to return current query string.
    pub query_string: Signal<String>,

    /// Signal to return the current full navigation url including query parameters.
    pub nav_url: Signal<String>,
}
