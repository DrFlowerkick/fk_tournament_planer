// single instance in memory implementation of ClientRegistryPort
// and some helper functions to integrate this in web services

mod leptos_hook;
#[cfg(feature = "ssr")]
mod registry;
mod types;
#[cfg(feature = "ssr")]
mod web_service;

pub use leptos_hook::*;
#[cfg(feature = "ssr")]
pub use registry::*;
pub use types::*;
#[cfg(feature = "ssr")]
pub use web_service::*;
