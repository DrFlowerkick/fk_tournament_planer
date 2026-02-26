//! state structures for the application

pub mod activity_tracker;
pub mod error_state;
pub mod global_state;
pub mod object_table;
pub mod postal_address;
pub mod sport_config;
pub mod toast_state;
pub mod tournament;

use app_core::utils::traits::ObjectIdVersion;
use leptos::prelude::*;
use uuid::Uuid;

pub trait EditorOptions {
    fn object_id(&self) -> Option<Uuid>;
}

pub trait EditorContext: Copy + Clone + Send + Sync + 'static {
    type ObjectType: ObjectIdVersion + Clone + Send + Sync + 'static;
    type NewEditorOptions: EditorOptions;

    /// Create a new instance of the editor context, initializing all necessary state.
    fn new(options: Self::NewEditorOptions) -> Self;
    /// Get the original object currently loaded in the editor context, if any.
    fn origin_signal(&self) -> Signal<Option<Self::ObjectType>>;
    /// Set the current object in the editor context, updating all relevant state accordingly.
    fn set_object(&self, object: Self::ObjectType);
    /// Create a new object based on the current state of the editor context, returning its unique identifier.
    fn new_object(&self) -> Option<Uuid>;
}

pub trait EditorContextWithResource: EditorContext {
    /// Create a new object from a given object by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, _object: Self::ObjectType) -> Option<Uuid>;
    /// Increment the optimistic version number in the editor context to handle concurrent edits.
    fn increment_optimistic_version(&self);
    /// If save fails, we need to reset the optimistic version to the origin version to prevent version mismatch on next save attempt.
    fn reset_version_to_origin(&self);
    /// Get the current optimistic version signal from the editor context, if any.
    fn optimistic_version_signal(&self) -> Signal<Option<u32>>;
}

#[derive(Clone, Copy)]
pub struct SimpleEditorOptions {
    pub object_id: Option<Uuid>,
}

impl EditorOptions for SimpleEditorOptions {
    fn object_id(&self) -> Option<Uuid> {
        self.object_id
    }
}

impl SimpleEditorOptions {
    pub fn with_id(object_id: Uuid) -> Self {
        Self {
            object_id: Some(object_id),
        }
    }

    pub fn no_id() -> Self {
        Self { object_id: None }
    }
}
