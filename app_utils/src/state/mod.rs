//! state structures for the application

pub mod activity_tracker;
pub mod error_state;
pub mod global_state;
pub mod object_table;
pub mod postal_address;
pub mod sport_config;
pub mod toast_state;
pub mod tournament_editor;

use app_core::utils::traits::ObjectIdVersion;
use leptos::prelude::*;
use uuid::Uuid;

pub trait EditorContext: Copy + Clone + Send + Sync + 'static {
    type ObjectType;

    /// Create a new instance of the editor context, initializing all necessary state.
    fn new() -> Self;
    /// Get the original object currently loaded in the editor context, if any.
    fn get_origin(&self) -> Option<Self::ObjectType>;
    /// Set the current object in the editor context, updating all relevant state accordingly.
    fn set_object(&self, object: Self::ObjectType);
    /// Create a new object based on the current state of the editor context, returning its unique identifier.
    fn new_object(&self) -> Option<Uuid>;
    /// Create a new object from a given object by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, _object: Self::ObjectType) -> Option<Uuid> {
        leptos::logging::warn!(
            "copy_object is not implemented for this editor context! This may result in unexpected \
            behavior if used in a context where copying is expected.\n\
            Please implement copy_object for this editor context to ensure correct functionality.\n\
            If this is not possible, consider implementing a custom copy mechanism in the relevant \
            component or logic instead."
        );
        None
    }
    /// Increment the optimistic version number in the editor context to handle concurrent edits.
    fn increment_optimistic_version(&self);
    /// If save fails, we need to reset the optimistic version to the origin version to prevent version mismatch on next save attempt.
    fn reset_version_to_origin(&self);
    /// Get the current optimistic version signal from the editor context, if any.
    fn get_optimistic_version(&self) -> Signal<Option<u32>>;
}

pub trait EditorContextWithObjectIdVersion:
    EditorContext<ObjectType = Self::ObjectTypeWithIdVersion>
{
    type ObjectTypeWithIdVersion: ObjectIdVersion + Clone;
}
