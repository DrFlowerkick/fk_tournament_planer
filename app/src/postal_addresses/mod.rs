// web ui for adding and modifying postal addresses

mod edit;
mod search;
pub mod server_fn;

#[cfg(feature = "test-mock")]
pub mod server_fn_test_support;

pub use edit::*;
pub use search::*;
