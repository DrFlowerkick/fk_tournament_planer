//! generic context for objects listed in tables

use crate::{
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::ParamQueryId,
    state::{EditorContext, EditorContextWithObjectIdVersion},
};
use app_core::utils::traits::ObjectIdVersion;
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQueryId,
{
    /// RwSignal for the list of objects to be displayed in the table
    pub object_list: RwSignal<Vec<O>>,
    /// Read slice for the currently selected object id
    pub selected_id: Signal<Option<Uuid>>,
    /// Callback for updating the currently selected object id
    pub set_selected_id: Callback<Option<Uuid>>,
    /// Trigger to refetch data from server
    refetch_trigger: RwSignal<u64>,
    /// Read slice for getting the current state of the object list
    pub track_fetch_trigger: Signal<u64>,
    // marker to keep generic type Q
    marker: std::marker::PhantomData<Q>,
}

impl<O, Q> Clone for ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQueryId,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<O, Q> Copy for ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQueryId,
{
}

impl<O, Q> ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQueryId,
{
    pub fn new() -> Self {
        let UseQueryNavigationReturn {
            url_update_query,
            url_remove_query,
            ..
        } = use_query_navigation();
        let navigate = use_navigate();

        let object_list = RwSignal::new(Vec::new());
        let selected_id_query = use_query::<Q>();
        let selected_id = Signal::derive(move || {
            selected_id_query.with(|qr| {
                qr.as_ref().ok().and_then(|q| {
                    q.get_id().and_then(|id| {
                        object_list.with(move |ol| {
                            ol.iter()
                                .any(|o: &O| o.get_id_version().get_id() == id)
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
                    url_update_query(Q::KEY, &t_id.to_string())
                } else {
                    url_remove_query(Q::KEY)
                };
                navigate(
                    &nav_url,
                    NavigateOptions {
                        replace: true,
                        scroll: false,
                        ..Default::default()
                    },
                );
            }
        });
        let refetch_trigger = RwSignal::new(0);

        Self {
            object_list,
            selected_id,
            set_selected_id,
            refetch_trigger,
            track_fetch_trigger: refetch_trigger.read_only().into(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn trigger_refetch(&self) {
        self.refetch_trigger.update(|v| *v += 1);
    }

    pub fn is_id_in_list(&self, id: Uuid) -> bool {
        self.object_list.with(|list| {
            list.iter()
                .any(|obj: &O| obj.get_id_version().get_id() == id)
        })
    }
}

pub struct ObjectEditorMapContext<OE, Q>
where
    OE: EditorContext,
    Q: ParamQueryId,
{
    /// RwSignal for the map of object editors
    editor_map: RwSignal<HashMap<Uuid, OE>>,
    /// RwSignal for the list of visible object editor ids
    pub visible_ids_list: RwSignal<Vec<Uuid>>,
    /// Read slice for the currently selected object editor id
    pub selected_id: Signal<Option<Uuid>>,
    /// Callback for updating the currently selected object editor id
    pub set_selected_id: Callback<Option<Uuid>>,
    /// Callback for creating a new object editor
    pub new_editor: Callback<(), Option<Uuid>>,
    /// Callback for copying the current object editor and selecting the copy
    pub copy_editor: Callback<(), Option<Uuid>>,
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
        let visible_ids_list = RwSignal::new(Vec::new());
        let selected_id_query = use_query::<Q>();
        let selected_id = Signal::derive(move || {
            selected_id_query.with(|qr| {
                qr.as_ref().ok().and_then(|q| {
                    q.get_id().and_then(|id| {
                        visible_ids_list.with(move |vids| vids.contains(&id).then_some(id))
                    })
                })
            })
        });
        let set_selected_id = Callback::new({
            let navigate = navigate.clone();
            move |new_id: Option<Uuid>| {
                let nav_url = if let Some(t_id) = new_id {
                    url_update_query(Q::KEY, &t_id.to_string())
                } else {
                    url_remove_query(Q::KEY)
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
        let new_editor = Callback::new(move |()| {
            let editor = OE::new();
            if let Some(new_id) = editor.new_object() {
                editor_map.update(|em| {
                    em.insert(new_id, editor);
                });
                Some(new_id)
            } else {
                None
            }
        });
        let copy_editor = Callback::new(move |()| {
            let editor = OE::new();
            if let Some(current_id) = selected_id.get()
                && let Some(origin) =
                    editor_map.with(|em| em.get(&current_id).and_then(|ed| ed.get_origin()))
                && let Some(new_id) = editor.copy_object(origin)
            {
                editor_map.update(|em| {
                    em.insert(new_id, editor);
                });
                Some(new_id)
            } else {
                None
            }
        });
        let refetch_trigger = RwSignal::new(0);

        Self {
            editor_map,
            visible_ids_list,
            selected_id,
            set_selected_id,
            new_editor,
            copy_editor,
            refetch_trigger,
            track_fetch_trigger: refetch_trigger.read_only().into(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn insert_editor(&self, id: Uuid, editor: OE) {
        self.editor_map.update(|em| {
            em.insert(id, editor);
        });
    }

    pub fn get_editor(&self, id: Uuid) -> Option<OE> {
        self.editor_map.with(|em| em.get(&id).copied())
    }

    pub fn get_editor_untracked(&self, id: Uuid) -> Option<OE> {
        self.editor_map.with_untracked(|em| em.get(&id).copied())
    }

    pub fn is_selected(&self, id: Uuid) -> bool {
        self.selected_id
            .with(|selected_id| selected_id == &Some(id))
    }

    pub fn remove_editor(&self, id: Uuid) {
        self.editor_map.update(|em| {
            em.remove(&id);
        });
    }

    pub fn trigger_refetch(&self) {
        self.refetch_trigger.update(|v| *v += 1);
    }
}

impl<OE, Q> ObjectEditorMapContext<OE, Q>
where
    OE: EditorContextWithObjectIdVersion,
    Q: ParamQueryId,
{
    pub fn update_object_in_editor(&self, object: &OE::ObjectTypeWithIdVersion) {
        self.editor_map.with(|em| {
            if let Some(editor) = em.get(&object.get_id_version().get_id()) {
                let optimistic_version = editor.get_optimistic_version().get();
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
}
