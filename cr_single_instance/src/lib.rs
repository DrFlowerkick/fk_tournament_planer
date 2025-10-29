// single instance in memory implementation of ClientRegistryPort
// and web service to use it with axum

#[cfg(feature = "ssr")]
pub mod registry;
mod types;

#[cfg(feature = "ssr")]
pub use registry::*;
pub use types::*;
