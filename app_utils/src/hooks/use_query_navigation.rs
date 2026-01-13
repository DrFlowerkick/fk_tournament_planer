//! Provides a hook for query-based navigation.

use leptos::prelude::*;
use leptos_router::{hooks::use_url, location::Url};

/// A hook that provides query-based navigation capabilities.
pub fn use_query_navigation() -> UseQueryNavigationReturn<
    impl Fn(&str) -> Option<String> + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str) + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str, Option<&str>) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, Option<&str>) -> String + Clone + Copy + Send + Sync + 'static,
> {
    let url = use_url();
    let mut_url = RwSignal::new(Url::default());
    Effect::new(move || mut_url.set(url.get()));
    let get = move |key: &str| mut_url.get().search_params().get(key);
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
    let url_with_param = move |key: &str, value: &str, sub_path: Option<&str>| {
        let mut new_url = mut_url.get();
        new_url
            .search_params_mut()
            .replace(key.to_string(), value.to_string());
        let qs = new_url.search_params().to_query_string();
        if let Some(path) = sub_path {
            format!("{}{}", path, qs)
        } else {
            format!("{}{}", new_url.path(), qs)
        }
    };
    let url_with_out_param = move |key: &str, sub_path: Option<&str>| {
        let mut new_url = mut_url.get();
        let _ = new_url.search_params_mut().remove(key);
        let qs = new_url.search_params().to_query_string();
        if let Some(path) = sub_path {
            format!("{}{}", path, qs)
        } else {
            format!("{}{}", new_url.path(), qs)
        }
    };

    UseQueryNavigationReturn {
        get,
        update,
        remove,
        path,
        query_string,
        nav_url,
        relative_sub_url,
        url_with_param,
        url_with_out_param,
    }
}

/// Return type of `use_query_navigation`.
pub struct UseQueryNavigationReturn<
    GetFn,
    UpdateFn,
    RemoveFn,
    RelativeSubUrlFn,
    UrlWithParamFn,
    UrlWithOutParamFn,
> where
    GetFn: Fn(&str) -> Option<String>,
    UpdateFn: Fn(&str, &str),
    RemoveFn: Fn(&str),
    RelativeSubUrlFn: Fn(&str) -> String,
    UrlWithParamFn: Fn(&str, &str, Option<&str>) -> String,
    UrlWithOutParamFn: Fn(&str, Option<&str>) -> String,
{
    /// Function to get the value of a query parameter by key.
    pub get: GetFn,

    /// Function to update a query in the query map.
    pub update: UpdateFn,

    /// Function to remove a key from the query map.
    pub remove: RemoveFn,

    /// Function which returns the current query string.
    /// Use this in combination with <A> component for relative sub-urls.
    pub relative_sub_url: RelativeSubUrlFn,

    /// Function to generate a URL with a specific query parameter updated.
    /// Optionally takes a sub_path to replace the current path.
    pub url_with_param: UrlWithParamFn,

    /// Function to generate a URL with a specific query parameter removed.
    /// Optionally takes a sub_path to replace the current path.
    pub url_with_out_param: UrlWithOutParamFn,

    /// Signal to return current path.
    pub path: Signal<String>,

    /// Signal to return current query string.
    pub query_string: Signal<String>,

    /// Signal to return the current full navigation url including query parameters.
    pub nav_url: Signal<String>,
}
