//! Provides a hook for query-based navigation.

use leptos::prelude::*;
use leptos_router::hooks::{use_matched, use_url};

#[derive(Clone, Copy)]
pub enum MatchedRouteHandler<'a> {
    Keep,
    RemoveSegment(u32),
    ReplaceSegment(&'a str),
    Extend(&'a str),
}

impl<'a> MatchedRouteHandler<'a> {
    /// Handle the matched route based on the specified strategy.
    pub fn handle(&self) -> String {
        let matched_route = use_matched();

        match self {
            MatchedRouteHandler::Keep => matched_route.get(),
            MatchedRouteHandler::RemoveSegment(count) => {
                let mut path = matched_route.get();
                for _ in 0..*count {
                    if let Some(pos) = path.rfind('/') {
                        path.truncate(pos);
                    }
                }
                if path.is_empty() {
                    path = "/".to_string();
                }
                path
            }
            MatchedRouteHandler::ReplaceSegment(new_segment) => {
                let mut path = matched_route.get();
                if let Some(pos) = path.rfind('/') {
                    path.truncate(pos);
                }
                if path.is_empty() {
                    return "/".to_string();
                }
                if new_segment.is_empty() {
                    path
                } else {
                    format!("{}/{}", path, new_segment)
                }
            }
            MatchedRouteHandler::Extend(sub_path) => {
                let mut mr = matched_route.get();
                if sub_path.is_empty() {
                    return mr;
                }
                if mr == "/" {
                    mr = "".to_string();
                }
                format!("{}/{}", mr, sub_path)
            }
        }
    }

    /// Handle the matched route based on the specified strategy.
    pub fn handle_untracked(&self) -> String {
        let matched_route = use_matched();

        match self {
            MatchedRouteHandler::Keep => matched_route.get_untracked(),
            MatchedRouteHandler::RemoveSegment(count) => {
                let mut path = matched_route.get_untracked();
                for _ in 0..*count {
                    if let Some(pos) = path.rfind('/') {
                        path.truncate(pos);
                    }
                }
                if path.is_empty() {
                    path = "/".to_string();
                }
                path
            }
            MatchedRouteHandler::ReplaceSegment(new_segment) => {
                let mut path = matched_route.get_untracked();
                if let Some(pos) = path.rfind('/') {
                    path.truncate(pos);
                }
                if path.is_empty() {
                    return "/".to_string();
                }
                if new_segment.is_empty() {
                    path
                } else {
                    format!("{}/{}", path, new_segment)
                }
            }
            MatchedRouteHandler::Extend(sub_path) => {
                let mut mr = matched_route.get_untracked();
                if sub_path.is_empty() {
                    return mr;
                }
                if mr == "/" {
                    mr = "".to_string();
                }
                format!("{}/{}", mr, sub_path)
            }
        }
    }
}

/// A hook that provides query-based navigation capabilities.
pub fn use_query_navigation() -> UseQueryNavigationReturn<
    impl Fn(&str) -> Option<String> + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(Vec<(&str, &str)>) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str, MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(Vec<(&str, &str)>, MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
> {
    let url = use_url();
    let get_query = move |key: &str| url.get().search_params().get(key);
    let url_update_query = move |key: &str, value: &str| {
        let mut new_url = url.get();
        new_url
            .search_params_mut()
            .replace(key.to_string(), value.to_string());
        format!(
            "{}{}",
            new_url.path(),
            new_url.search_params().to_query_string()
        )
    };
    let url_update_queries = move |key_value: Vec<(&str, &str)>| {
        let mut new_url = url.get();
        for (key, value) in key_value {
            new_url
                .search_params_mut()
                .replace(key.to_string(), value.to_string());
        }
        format!(
            "{}{}",
            new_url.path(),
            new_url.search_params().to_query_string()
        )
    };
    let url_remove_query = move |key: &str| {
        let mut new_url = url.get();
        let _ = new_url.search_params_mut().remove(key);
        format!(
            "{}{}",
            new_url.path(),
            new_url.search_params().to_query_string()
        )
    };
    let url_matched_route = move |matched_route_handler: MatchedRouteHandler| {
        format!(
            "{}{}",
            matched_route_handler.handle(),
            url.get().search_params().to_query_string()
        )
    };
    let url_matched_route_update_query =
        move |key: &str, value: &str, matched_route_handler: MatchedRouteHandler| {
            let mut new_url = url.get();
            new_url
                .search_params_mut()
                .replace(key.to_string(), value.to_string());
            format!(
                "{}{}",
                matched_route_handler.handle(),
                new_url.search_params().to_query_string()
            )
        };
    let url_matched_route_update_queries =
        move |key_value: Vec<(&str, &str)>, matched_route_handler: MatchedRouteHandler| {
            let mut new_url = url.get();
            for (key, value) in key_value {
                new_url
                    .search_params_mut()
                    .replace(key.to_string(), value.to_string());
            }
            format!(
                "{}{}",
                matched_route_handler.handle(),
                new_url.search_params().to_query_string()
            )
        };
    let url_matched_route_remove_query =
        move |key: &str, matched_route_handler: MatchedRouteHandler| {
            let mut new_url = url.get();
            let _ = new_url.search_params_mut().remove(key);
            format!(
                "{}{}",
                matched_route_handler.handle(),
                new_url.search_params().to_query_string()
            )
        };
    let matched_path = use_matched();
    let url_is_matched_route = Memo::new(move |_| url.get().path() == matched_path.get());
    UseQueryNavigationReturn {
        get_query,
        url_update_query,
        url_update_queries,
        url_remove_query,
        url_matched_route,
        url_matched_route_update_query,
        url_matched_route_update_queries,
        url_matched_route_remove_query,
        url_is_matched_route,
    }
}

/// Return type of `use_query_navigation`.
pub struct UseQueryNavigationReturn<
    GetFn,
    UrlUpdateQueryFn,
    UrlUpdateQueriesFn,
    UrlRemoveQueryFn,
    UrlMatchedRouteFn,
    UrlMatchedRouteUpdateQueryFn,
    UrlMatchedRouteUpdateQueriesFn,
    UrlMatchedRouteRemoveQueryFn,
> where
    GetFn: Fn(&str) -> Option<String>,
    UrlUpdateQueryFn: Fn(&str, &str) -> String,
    UrlUpdateQueriesFn: Fn(Vec<(&str, &str)>) -> String,
    UrlRemoveQueryFn: Fn(&str) -> String,
    UrlMatchedRouteFn: Fn(MatchedRouteHandler) -> String,
    UrlMatchedRouteUpdateQueryFn: Fn(&str, &str, MatchedRouteHandler) -> String,
    UrlMatchedRouteUpdateQueriesFn: Fn(Vec<(&str, &str)>, MatchedRouteHandler) -> String,
    UrlMatchedRouteRemoveQueryFn: Fn(&str, MatchedRouteHandler) -> String,
{
    /// Function to get the value of a query parameter by key.
    pub get_query: GetFn,

    /// Function to return current url with an updated specific query parameter.
    pub url_update_query: UrlUpdateQueryFn,

    /// Function to return current url with an updated multiple query parameter.
    pub url_update_queries: UrlUpdateQueriesFn,

    /// Function to return current url with a specific query parameter removed.
    pub url_remove_query: UrlRemoveQueryFn,

    /// Function to generate a URL based on the matched route.
    /// The matched route may be kept, have segments removed, or be extended.
    pub url_matched_route: UrlMatchedRouteFn,

    /// Same as `url_matched_route`, but also updates a specific query parameter.
    pub url_matched_route_update_query: UrlMatchedRouteUpdateQueryFn,

    /// Same as `url_matched_route`, but also updates multiple query parameter.
    pub url_matched_route_update_queries: UrlMatchedRouteUpdateQueriesFn,

    /// Same as `url_matched_route`, but removes a specific query parameter.
    pub url_matched_route_remove_query: UrlMatchedRouteRemoveQueryFn,

    /// Memo to check if the current URL matches the route of the router.
    pub url_is_matched_route: Memo<bool>,
}
