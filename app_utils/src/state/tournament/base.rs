//! base editor context

use super::TournamentEditorContext;
use crate::state::EditorContext;
use leptos::prelude::*;

struct BaseEditorContext {}

impl BaseEditorContext {
    fn test() {
        let tournament_editor_context = TournamentEditorContext::new(());
        let local = tournament_editor_context.local;
    }
}
