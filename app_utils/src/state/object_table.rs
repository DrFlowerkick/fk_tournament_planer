//! generic context for objects listed in tables

use crate::{
    hooks::use_url_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::ParamQueryId,
    state::{EditorContext, EditorContextWithResource, EditorOptions},
};
use app_core::utils::traits::ObjectIdVersion;
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ObjectEditorMapContext<OE, Q>
where
    OE: EditorContext,
    Q: ParamQueryId,
{
    /// RwSignal for the map of object editors
    editor_map: RwSignal<HashMap<Uuid, (OE, Owner)>>,
    /// Owner where the context is provided, used for creating new signals in the context of the editors
    pub owner: StoredValue<Owner>,
    /// RwSignal for the list of visible object editor ids
    // ToDo: remove this after refactoring of list management
    pub visible_ids_list: RwSignal<Vec<Uuid>>,
    /// List of visible objects, loaded from the server
    pub visible_objects_list: RwSignal<Vec<OE::ObjectType>>,
    /// Read slice for the currently selected object editor id
    pub selected_id: Signal<Option<Uuid>>,
    /// Callback for updating the currently selected object editor id
    pub set_selected_id: Callback<Option<Uuid>>,
    /// Trigger to refetch data from server
    refetch_trigger: RwSignal<u64>,
    /// Read slice for getting the current state of the object editor map
    pub track_fetch_trigger: Signal<u64>,
    // marker to keep generic type Q
    marker: std::marker::PhantomData<Q>,
}

impl<OE, Q> Clone for ObjectEditorMapContext<OE, Q>
where
    OE: EditorContext,
    Q: ParamQueryId,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<OE, Q> Copy for ObjectEditorMapContext<OE, Q>
where
    OE: EditorContext,
    Q: ParamQueryId,
{
}

impl<OE, Q> ObjectEditorMapContext<OE, Q>
where
    OE: EditorContext,
    Q: ParamQueryId,
{
    pub fn new() -> Self {
        let UseQueryNavigationReturn {
            url_update_query,
            url_remove_query,
            ..
        } = use_query_navigation();
        let navigate = use_navigate();

        let editor_map = RwSignal::new(HashMap::new());
        let owner = StoredValue::new(Owner::current().expect("No reactive owner found"));
        let visible_ids_list = RwSignal::new(Vec::new());
        let visible_objects_list = RwSignal::new(Vec::new());
        let selected_id_query = use_query::<Q>();
        let selected_id = Signal::derive(move || {
            selected_id_query.with(|qr| {
                qr.as_ref().ok().and_then(|q| {
                    q.get_id().and_then(|id| {
                        visible_objects_list.with(move |vos| {
                            vos.iter()
                                .any(|vo: &OE::ObjectType| vo.get_id_version().get_id() == id)
                                .then_some(id)
                        })
                    })
                })
            })
        });
        let set_selected_id = Callback::new({
            let navigate = navigate.clone();
            move |new_id: Option<Uuid>| {
                let nav_url = if let Some(t_id) = new_id {
                    url_update_query(Q::KEY, &t_id.to_string(), None)
                } else {
                    url_remove_query(Q::KEY, None)
                };
                navigate(
                    &nav_url,
                    NavigateOptions {
                        scroll: false,
                        ..Default::default()
                    },
                );
            }
        });
        let refetch_trigger = RwSignal::new(0);

        Self {
            editor_map,
            owner,
            visible_ids_list,
            visible_objects_list,
            selected_id,
            set_selected_id,
            refetch_trigger,
            track_fetch_trigger: refetch_trigger.read_only().into(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn spawn_editor_for_new_object(&self, options: OE::NewEditorOptions) -> Option<OE> {
        let child = self.owner.get_value().child();

        // Execute creation inside the child scope
        let new_context = child.with(|| {
            let editor = OE::new(options);
            editor.new_object().map(|new_id| (new_id, editor))
        });
        if let Some((new_id, editor)) = new_context {
            self.editor_map.update(|em| {
                em.insert(new_id, (editor, child));
            });
            Some(editor)
        } else {
            child.cleanup();
            None
        }
    }

    pub fn spawn_editor_for_edit_object(&self, options: OE::NewEditorOptions) -> Option<OE> {
        let Some(object_id) = options.object_id() else {
            // Cannot edit without an object ID
            return None;
        };

        if let Some(editor) = self
            .editor_map
            .with_untracked(|em| em.get(&object_id).map(|(editor, _)| *editor))
        {
            // Editor already exists for this object ID
            return Some(editor);
        }

        let child = self.owner.get_value().child();

        // Execute creation inside the child scope
        let editor = child.with(|| OE::new(options));
        self.editor_map.update(|em| {
            em.insert(object_id, (editor, child));
        });
        Some(editor)
    }

    pub fn get_editor(&self, id: Uuid) -> Option<OE> {
        self.editor_map
            .try_with(|em| em.get(&id).map(|(editor, _)| *editor))
            .flatten()
    }

    pub fn get_editor_untracked(&self, id: Uuid) -> Option<OE> {
        self.editor_map
            .with_untracked(|em| em.get(&id).map(|(editor, _)| *editor))
    }

    pub fn remove_editor(&self, id: Uuid) {
        self.editor_map.update(|em| {
            if let Some((_, child)) = em.remove(&id) {
                child.cleanup();
            }
        });
    }

    pub fn remove_all(&self) {
        self.editor_map.update(|em| {
            for child in em.drain().map(|(_, (_, child))| child) {
                child.cleanup();
            }
        });
    }

    pub fn is_selected(&self, id: Uuid) -> bool {
        self.selected_id
            .with(|selected_id| selected_id == &Some(id))
    }
}

impl<OE, Q> ObjectEditorMapContext<OE, Q>
where
    OE: EditorContextWithResource,
    Q: ParamQueryId,
{
    pub fn spawn_editor_for_copy_object(
        &self,
        source_id: Uuid,
        options: OE::NewEditorOptions,
    ) -> Option<OE> {
        let Some(source) = self.editor_map.with(|em| {
            em.get(&source_id)
                .and_then(|(ed, _)| ed.get_versioned_object())
        }) else {
            // Cannot copy without source object data
            return None;
        };

        let child = self.owner.get_value().child();

        // Execute creation inside the child scope
        let new_context = child.with(|| {
            let editor = OE::new(options);
            editor.copy_object(source).map(|new_id| (new_id, editor))
        });
        if let Some((new_id, editor)) = new_context {
            self.editor_map.update(|em| {
                em.insert(new_id, (editor, child));
            });
            Some(editor)
        } else {
            child.cleanup();
            None
        }
    }

    // ToDo: we have to change all signal usage in this fn to _untracked
    pub fn update_object_in_editor(&self, object: &OE::ObjectType) {
        self.editor_map.with(|em| {
            if let Some(editor) = em
                .get(&object.get_id_version().get_id())
                .map(|(editor, _)| editor)
            {
                let optimistic_version = editor.optimistic_version_signal().get();
                if optimistic_version.is_none() {
                    editor.set_object(object.clone());
                }
                if let Some(ov) = optimistic_version
                    && ov < object.get_id_version().get_version().unwrap_or_default()
                {
                    editor.set_object(object.clone());
                }
            }
        });
    }

    pub fn trigger_refetch(&self) {
        self.refetch_trigger.update(|v| *v += 1);
    }
}
