//! Provides a hook for query-based navigation.

use leptos::prelude::*;
use leptos_router::hooks::{use_matched, use_url};

#[derive(Clone, Copy)]
pub enum MatchedRouteHandler<'a> {
    Keep,
    RemoveSegment(u32),
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
            MatchedRouteHandler::Extend(sub_path) => {
                let mut mr = matched_route.get();
                if *sub_path == "" {
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
            MatchedRouteHandler::Extend(sub_path) => {
                let mut mr = matched_route.get_untracked();
                if *sub_path == "" {
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
    impl Fn(MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, &str, MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
    impl Fn(&str, MatchedRouteHandler) -> String + Clone + Copy + Send + Sync + 'static,
> {
    let url = use_url();
    let get_query = move |key: &str| url.get().search_params().get(key);
    let path = Signal::derive(move || url.get().path().to_string());
    let query_string = Signal::derive(move || url.get().search_params().to_query_string());
    let nav_url = Signal::derive(move || format!("{}{}", path.get(), query_string.get()));
    let url_matched_route = move |matched_route_handler: MatchedRouteHandler| {
        format!("{}{}", matched_route_handler.handle(), query_string.get())
    };
    let url_matched_route_update_query =
        move |key: &str, value: &str, matched_route_handler: MatchedRouteHandler| {
            let mut new_url = url.get();
            new_url
                .search_params_mut()
                .replace(key.to_string(), value.to_string());
            let qs = new_url.search_params().to_query_string();
            format!("{}{}", matched_route_handler.handle(), qs)
        };
    let url_matched_route_remove_query =
        move |key: &str, matched_route_handler: MatchedRouteHandler| {
            let mut new_url = url.get();
            let _ = new_url.search_params_mut().remove(key);
            let qs = new_url.search_params().to_query_string();
            format!("{}{}", matched_route_handler.handle(), qs)
        };

    UseQueryNavigationReturn {
        get_query,
        path,
        query_string,
        nav_url,
        url_matched_route,
        url_matched_route_update_query,
        url_matched_route_remove_query,
    }
}

/// Return type of `use_query_navigation`.
pub struct UseQueryNavigationReturn<
    GetFn,
    UrlMatchedRouteFn,
    UrlMatchedRouteUpdateQueryFn,
    UrlMatchedRouteRemoveQueryFn,
> where
    GetFn: Fn(&str) -> Option<String>,
    UrlMatchedRouteFn: Fn(MatchedRouteHandler) -> String,
    UrlMatchedRouteUpdateQueryFn: Fn(&str, &str, MatchedRouteHandler) -> String,
    UrlMatchedRouteRemoveQueryFn: Fn(&str, MatchedRouteHandler) -> String,
{
    /// Function to get the value of a query parameter by key.
    pub get_query: GetFn,

    /// Function which adds provided sub path to matched route and current query string.
    /// provided sub path must be relative, meaning it must NOT start with a '/'.
    /// Use this in combination with <A> component, if router panics occur with
    /// 'url_with_path' and relative paths due to nested routes.
    pub url_matched_route: UrlMatchedRouteFn,

    /// Function to generate a URL with a specific query parameter updated.
    /// Optionally takes a sub_path to replace the current path.
    pub url_matched_route_update_query: UrlMatchedRouteUpdateQueryFn,

    /// Function to generate a URL with a specific query parameter removed.
    /// Optionally takes a sub_path to replace the current path.
    pub url_matched_route_remove_query: UrlMatchedRouteRemoveQueryFn,

    /// Signal to return current path.
    pub path: Signal<String>,

    /// Signal to return current query string.
    pub query_string: Signal<String>,

    /// Signal to return the current full navigation url including query parameters.
    pub nav_url: Signal<String>,
}
