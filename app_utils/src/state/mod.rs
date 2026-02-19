//! state structures for the application

pub mod activity_tracker;
pub mod error_state;
pub mod global_state;
pub mod object_table_list;
pub mod postal_address;
pub mod sport_config;
pub mod toast_state;
pub mod tournament_editor;

use leptos::prelude::*;

pub trait EditorContext: Copy + Clone + Send + Sync + 'static {
    fn has_origin(&self) -> Signal<bool>;
    fn has_id(&self) -> Signal<bool>;
    fn prepare_copy(&self);
    fn new_object(&self);
}
