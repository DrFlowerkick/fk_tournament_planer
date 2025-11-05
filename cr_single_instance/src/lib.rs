// single instance in memory implementation of ClientRegistryPort
// and web service to use it with axum

mod leptos_hook;
#[cfg(feature = "ssr")]
mod registry;
#[cfg(all(feature = "ssr", any(test, feature = "test_support")))]
pub mod test_support;
mod types;
#[cfg(feature = "ssr")]
mod web_service;

pub use leptos_hook::*;
#[cfg(feature = "ssr")]
pub use registry::*;
pub use types::*;
#[cfg(feature = "ssr")]
pub use web_service::*;
