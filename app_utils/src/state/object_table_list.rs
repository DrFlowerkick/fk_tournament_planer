//! generic context for objects listed in tables

use crate::{
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::ParamQuery,
};
use app_core::utils::traits::ObjectIdVersion;
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

pub struct ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQuery<Uuid>,
{
    /// StoredValue for the list of objects to be displayed in the table
    pub object_list: StoredValue<Vec<O>>,
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
    Q: ParamQuery<Uuid>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<O, Q> Copy for ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQuery<Uuid>,
{
}

impl<O, Q> ObjectListContext<O, Q>
where
    O: ObjectIdVersion + Send + Sync + 'static,
    Q: ParamQuery<Uuid>,
{
    pub fn new() -> Self {
        let UseQueryNavigationReturn {
            url_update_query,
            url_remove_query,
            ..
        } = use_query_navigation();
        let navigate = use_navigate();

        let object_list = StoredValue::new(Vec::new());
        let selected_id = Q::use_param_query();
        let set_selected_id = Callback::new({
            let navigate = navigate.clone();
            move |new_id: Option<Uuid>| {
                let nav_url = if let Some(t_id) = new_id {
                    url_update_query(Q::key(), &t_id.to_string())
                } else {
                    url_remove_query(Q::key())
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
        self.object_list.with_value(|list| {
            list.iter()
                .any(|obj: &O| obj.get_id_version().get_id() == id)
        })
    }
}
