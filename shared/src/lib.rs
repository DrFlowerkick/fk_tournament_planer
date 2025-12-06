// shared data types, used by server and/ or client

#[cfg(feature = "hydrate")]
mod client;
mod client_and_server;
#[cfg(feature = "ssr")]
mod server;

#[allow(unused_imports)] // currently there is no shared code on client only
#[cfg(feature = "hydrate")]
pub use client::*;
pub use client_and_server::*;
#[cfg(feature = "ssr")]
pub use server::*;
