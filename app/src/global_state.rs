//! global state management in reactive system of app

use reactive_stores::Store;

#[derive(Clone, Debug, Default, Store)]
pub struct GlobalState {
    /// path before new/edit address
    pub return_after_address_edit: String,
}
