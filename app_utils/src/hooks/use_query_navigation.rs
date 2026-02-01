//! Provides a hook for query-based navigation.

use leptos::prelude::*;
use leptos_router::hooks::{use_matched, use_url};

/// A hook that provides query-based navigation capabilities.
pub fn use_query_navigation() -> UseQueryNavigationReturn<
    impl Fn(&str) -> Option<String> + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str, Option<&str>) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, Option<&str>) -> String + Clone + Copy + Send + Sync + 'static,
> {
    let url = use_url();
    let matched_route = use_matched();
    let get_query = move |key: &str| url.get().search_params().get(key);
    let path = Signal::derive(move || url.get().path().to_string());
    let query_string = Signal::derive(move || url.get().search_params().to_query_string());
    let nav_url = Signal::derive(move || format!("{}{}", path.get(), query_string.get()));
    let url_with_path = move |path: &str| format!("{}{}", path, query_string.get());
    let url_route_with_sub_path = move |sub_path: &str| {
        let mut mr = matched_route.get();
        if sub_path.is_empty() {
            return format!("{}{}", mr, query_string.get());
        }
        if mr == "/" {
            mr = "".to_string();
        }
        format!("{}/{}{}", mr, sub_path, query_string.get())
    };
    let url_with_update_query = move |key: &str, value: &str, sub_path: Option<&str>| {
        let mut new_url = url.get();
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
    let url_with_remove_query = move |key: &str, sub_path: Option<&str>| {
        let mut new_url = url.get();
        let _ = new_url.search_params_mut().remove(key);
        let qs = new_url.search_params().to_query_string();
        if let Some(path) = sub_path {
            format!("{}{}", path, qs)
        } else {
            format!("{}{}", new_url.path(), qs)
        }
    };

    UseQueryNavigationReturn {
        get_query,
        matched_route,
        path,
        query_string,
        nav_url,
        url_with_path,
        url_route_with_sub_path,
        url_with_update_query,
        url_with_remove_query,
    }
}

/// Return type of `use_query_navigation`.
pub struct UseQueryNavigationReturn<GetFn, UrlPathFn, UrlRouteSubPathFn, UrlUpdateFn, UrlRemoveFn>
where
    UrlPathFn: Fn(&str) -> String,
    UrlRouteSubPathFn: Fn(&str) -> String,
    UrlUpdateFn: Fn(&str, &str, Option<&str>) -> String,
    UrlRemoveFn: Fn(&str, Option<&str>) -> String,
{
    /// Function to get the value of a query parameter by key.
    pub get_query: GetFn,

    /// Function which returns a url with input path and current query string.
    /// Path may be relative.
    /// Use relative path in combination with <A> component for relative sub-urls.
    pub url_with_path: UrlPathFn,

    /// Function which adds provided sub path to matched route and current query string.
    /// provided sub path must be relative, meaning it must NOT start with a '/'.
    /// Use this in combination with <A> component, if router panics occur with
    /// 'url_with_path' and relative paths due to nested routes.
    pub url_route_with_sub_path: UrlRouteSubPathFn,

    /// Function to generate a URL with a specific query parameter updated.
    /// Optionally takes a sub_path to replace the current path.
    pub url_with_update_query: UrlUpdateFn,

    /// Function to generate a URL with a specific query parameter removed.
    /// Optionally takes a sub_path to replace the current path.
    pub url_with_remove_query: UrlRemoveFn,

    /// Memo to matched route
    pub matched_route: Memo<String>,

    /// Signal to return current path.
    pub path: Signal<String>,

    /// Signal to return current query string.
    pub query_string: Signal<String>,

    /// Signal to return the current full navigation url including query parameters.
    pub nav_url: Signal<String>,
}
