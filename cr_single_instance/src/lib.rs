// single instance in memory implementation of ClientRegistryPort
// and some helper functions to integrate this in web services

#[cfg(any(feature = "hydrate", feature = "ssr"))]
mod leptos_hook;
#[cfg(feature = "ssr")]
mod registry;
#[cfg(feature = "types")]
mod types;
#[cfg(feature = "ssr")]
mod web_service;

#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use leptos_hook::*;
#[cfg(feature = "ssr")]
pub use registry::*;
#[cfg(feature = "types")]
pub use types::*;
#[cfg(feature = "ssr")]
pub use web_service::*;
