// single instance in memory implementation of ClientRegistryPort
// and web service to use it with axum

#[cfg(feature = "ssr")]
mod registry;
mod types;
#[cfg(feature = "ssr")]
mod web_service;

#[cfg(feature = "ssr")]
pub use registry::*;
pub use types::*;
#[cfg(feature = "ssr")]
pub use web_service::*;
