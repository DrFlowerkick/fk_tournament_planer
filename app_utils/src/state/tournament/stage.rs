//! stage editor context

use crate::{
    error::{AppError, strategy::handle_with_toast},
    server_fn::stage::{LoadStageById, SaveStage},
    state::{
        EditorContext, EditorContextWithResource, EditorOptions, LabeledAction,
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        toast_state::ToastContext,
    },
};
use app_core::{
    CrTopic, Stage, Tournament, TournamentState,
    utils::{id_version::IdVersion, validation::ValidationResult},
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::prelude::*;
use uuid::Uuid;

pub struct StageEditorContextOptions {
    pub stage_number: u32,
    pub object_id: Option<Uuid>,
    pub local_tournament: RwSignal<Option<Tournament>>,
}

impl EditorOptions for StageEditorContextOptions {
    fn object_id(&self) -> Option<Uuid> {
        self.object_id
    }
}

#[derive(Clone, Copy)]
pub struct StageEditorContext {
    /// The predefined stage number in the editor context.
    stage_number: u32,

    // --- state & derived signals ---
    /// The local editable stage in the editor context, derived from the local tournament and stage number.
    pub local: Signal<Option<Stage>>,
    /// SignalSetter for setting the local stage in the editor context
    set_local: SignalSetter<Option<Stage>>,
    /// Read slice for accessing the validation result of the stage
    pub validation_result: Signal<ValidationResult<()>>,
    /// Read slice for checking if the stage is in a state where editing is disabled
    /// (e.g. when stage or tournament is finished)
    pub is_disabled_stage_editing: Signal<bool>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the id field
    pub id: Signal<Option<Uuid>>,
    /// Signal slice for the version field
    pub version: Signal<Option<u32>>,
    /// Signal slice for the tournament ID field
    pub tournament_id: Signal<Option<Uuid>>,
    /// Signal slice for the number field
    pub number: Signal<Option<u32>>,
    /// Read slice for accessing the stage number of groups, if any
    pub num_groups: Signal<Option<u32>>,
    /// Write slice for setting the stage number of groups
    pub set_num_groups: Callback<Option<u32>>,
    /// Signal for the group sizes of the stage, which is derived from the local stage and number of groups
    pub group_sizes: Signal<Vec<u32>>,
    /// Callback for setting the group size of a specific group, which updates the local stage accordingly
    pub set_group_sizes: Callback<(usize, u32)>,

    // --- Resource & server action state ---
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    set_optimistic_version: RwSignal<Option<u32>>,
    /// Server action for saving the stage based on the current state of the editor context
    pub save_stage: ServerAction<SaveStage>,
    /// Callback after successful save to e.g. navigate to the new stage or show a success toast.
    pub post_save_callback: StoredValue<Option<Callback<Stage>>>,
}

impl EditorContext for StageEditorContext {
    type ObjectType = Stage;
    type NewEditorOptions = StageEditorContextOptions;

    fn new(options: Self::NewEditorOptions) -> Self {
        // ---- global state & context ----
        let toast_ctx = expect_context::<ToastContext>();
        let page_err_ctx = expect_context::<PageErrorContext>();
        let activity_tracker = expect_context::<ActivityTracker>();
        let component_id = StoredValue::new(Uuid::new_v4());
        // remove errors on unmount
        on_cleanup(move || {
            page_err_ctx.clear_all_for_component(component_id.get_value());
            activity_tracker.remove_component(component_id.get_value());
        });

        // ---- signals & slices ----
        // get id first, which enables look up of stage by id, which is faster than look up by stage number.
        let id = create_read_slice(options.local_tournament, move |local_tournament| {
            local_tournament.as_ref().and_then(|t| {
                t.get_stage_by_number(options.stage_number)
                    .map(|s| s.get_id())
            })
        });

        let (local, set_local) = create_slice(
            options.local_tournament,
            move |local_tournament| {
                id.get().and_then(|id| {
                    local_tournament
                        .as_ref()
                        .and_then(|t| t.get_stage_by_id(id).cloned())
                })
            },
            |local_tournament, new_stage: Option<Stage>| {
                if let Some(new_stage) = new_stage
                    && let Some(t) = local_tournament
                {
                    t.set_stage(new_stage);
                }
            },
        );
        let validation_result =
            create_read_slice(options.local_tournament, move |local_tournament| {
                if let Some(id) = id.get()
                    && let Some(t) = local_tournament
                    && let Some(stage) = t.get_stage_by_id(id)
                {
                    stage.validate(t.get_base())
                } else {
                    ValidationResult::Ok(())
                }
            });

        let is_disabled_stage_editing =
            create_read_slice(options.local_tournament, move |local_tournament| {
                if let Some(t) = local_tournament {
                    match t.get_base().get_tournament_state() {
                        TournamentState::ActiveStage(active_stage) => {
                            active_stage >= options.stage_number
                        }
                        TournamentState::Finished => true,
                        _ => false,
                    }
                } else {
                    false
                }
            });

        let tournament_id = create_read_slice(options.local_tournament, |local_tournament| {
            local_tournament.as_ref().map(|t| t.get_base().get_id())
        });

        let version = create_read_slice(options.local_tournament, move |local_tournament| {
            id.get().and_then(|id| {
                local_tournament
                    .as_ref()
                    .and_then(|t| t.get_stage_by_id(id))
                    .and_then(|s| s.get_version())
            })
        });

        let number = create_read_slice(options.local_tournament, move |local_tournament| {
            id.get().and_then(|id| {
                local_tournament
                    .as_ref()
                    .and_then(|t| t.get_stage_by_id(id))
                    .map(|s| s.get_number())
            })
        });
        let (num_groups, set_num_groups) = create_slice(
            options.local_tournament,
            move |local_tournament| {
                id.get().and_then(|id| {
                    local_tournament
                        .as_ref()
                        .and_then(|t| t.get_stage_by_id(id))
                        .map(|s| s.get_number_of_groups())
                })
            },
            move |local_tournament, num_groups: u32| {
                if let Some(id) = id.get()
                    && let Some(t) = local_tournament
                {
                    t.set_stage_number_of_groups(id, num_groups);
                }
            },
        );
        let set_num_groups = Callback::new(move |num_groups: Option<u32>| {
            set_num_groups.set(num_groups.unwrap_or_default());
        });
        let (group_sizes, set_group_sizes) = create_slice(
            options.local_tournament,
            move |local_tournament| {
                if let Some(id) = id.get()
                    && let Some(t) = local_tournament
                    && let Some(stage) = t.get_stage_by_id(id)
                {
                    stage.get_group_sizes().into()
                } else {
                    vec![]
                }
            },
            move |local_tournament, (group_index, group_size): (usize, u32)| {
                if let Some(id) = id.get()
                    && let Some(t) = local_tournament
                {
                    t.set_stage_group_size(id, group_index, group_size);
                }
            },
        );
        let set_group_sizes = Callback::new(move |(group_index, group_size): (usize, u32)| {
            set_group_sizes.set((group_index, group_size));
        });

        // ---- tournament stage server action ----
        let (resource_id, set_resource_id) = signal(options.object_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // server action to fetch updated tournament stage for the given id, used by client registry
        let fetch_tournament_stage = ServerAction::<LoadStageById>::new();
        let fetch_tournament_stage_pending = fetch_tournament_stage.pending();
        activity_tracker
            .track_pending_memo(component_id.get_value(), fetch_tournament_stage_pending);

        let refetch = Callback::new(move |()| {
            if let Some(tournament_id) = tournament_id.get()
                && let Some(id) = resource_id.get()
            {
                fetch_tournament_stage.dispatch(LoadStageById { tournament_id, id });
            }
        });

        let topic =
            Signal::derive(move || resource_id.get().map(|id| CrTopic::Stage { stage_id: id }));
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // handle fetch result
        Effect::new(move || {
            if let Some(fetch_result) = fetch_tournament_stage.value().get() {
                fetch_tournament_stage.clear();
                match fetch_result {
                    Ok(Some(tb)) => {
                        set_resource_id.set(Some(tb.get_id()));
                        set_optimistic_version.set(tb.get_version());
                        set_local.set(Some(tb));
                    }
                    Ok(None) => {
                        // This case should not happen, since the fetch action is triggered based on the presence of a valid
                        // resource id. If it does happen, it means the resource was not found and we should inform the user.
                        let err = AppError::ResourceNotFound(
                            "Tournament Stage".to_string(),
                            resource_id.get().unwrap_or_default(),
                        );
                        handle_with_toast(&toast_ctx, &err, None);
                    }
                    Err(err) => {
                        let interactive = LabeledAction {
                            label: "Retry".to_string(),
                            on_click: refetch,
                        };
                        handle_with_toast(&toast_ctx, &err, Some(interactive));
                    }
                }
            }
        });

        // server action for saving the tournament stage based on the current state of the editor context
        let save_stage = ServerAction::<SaveStage>::new();
        let save_stage_pending = save_stage.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), save_stage_pending);

        let post_save_callback = StoredValue::new(None::<Callback<Stage>>);

        // handle save result
        Effect::new(move || {
            if let Some(stb_result) = save_stage.value().get() {
                save_stage.clear();
                match stb_result {
                    Ok(tb) => {
                        set_resource_id.set(Some(tb.get_id()));
                        set_optimistic_version.set(tb.get_version());
                        set_local.set(Some(tb.clone()));

                        if let Some(callback) = post_save_callback.get_value() {
                            callback.run(tb);
                        }
                    }
                    Err(err) => {
                        // version reset for parallel editing
                        set_optimistic_version.set(version.get());
                        handle_with_toast(&toast_ctx, &err, None);
                    }
                }
            }
        });

        StageEditorContext {
            stage_number: options.stage_number,
            local,
            set_local,
            validation_result,
            is_disabled_stage_editing,
            id,
            version,
            tournament_id,
            number,
            num_groups,
            set_num_groups,
            group_sizes,
            set_group_sizes,
            set_optimistic_version,
            save_stage,
            post_save_callback,
        }
    }

    /// Set an existing tournament stage in the editor context.
    fn set_object(&self, stage: Self::ObjectType) {
        self.set_local.set(Some(stage.clone()));
        self.set_optimistic_version.set(stage.get_version());
    }

    /// Create a new tournament stage in the editor context with a new UUID and default values.
    fn new_object(&self) -> Option<Uuid> {
        if let Some(tournament_id) = self.tournament_id.get() {
            let mut stage = Stage::default();
            stage
                .set_number(self.stage_number)
                .set_tournament_id(tournament_id);

            let id = stage.get_id();

            self.set_local.set(Some(stage));
            self.set_optimistic_version.set(None);
            Some(id)
        } else {
            None
        }
    }
}

impl EditorContextWithResource for StageEditorContext {
    /// Get the current tournament stage in the editor context with its version, if any.
    fn get_versioned_object(&self) -> Option<Self::ObjectType> {
        self.local.with(|local| {
            local
                .as_ref()
                .and_then(|ts| ts.get_version().map(|_| ts.clone()))
        })
    }

    /// Create a new object from a given tournament stage by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, mut stage: Self::ObjectType) -> Option<Uuid> {
        if let Some(tournament_id) = self.tournament_id.get()
            && self.stage_number == stage.get_number()
        {
            let id = Uuid::new_v4();
            stage
                .set_id_version(IdVersion::new(id, None))
                .set_tournament_id(tournament_id);
            self.set_local.set(Some(stage));
            self.set_optimistic_version.set(None);
            Some(id)
        } else {
            None
        }
    }

    /// Increment the optimistic version in the editor context to optimistically handle version updates after saving.
    fn increment_optimistic_version(&self) {
        self.set_optimistic_version.update(|v| {
            if let Some(current_version) = v {
                *current_version += 1
            } else {
                *v = Some(0)
            }
        });
    }

    /// If save fails, we need to reset the version to the original version to prevent version mismatch on next save attempt.
    fn reset_version_to_origin(&self) {
        self.set_optimistic_version.set(self.version.get());
    }

    /// Get the current optimistic version signal from the editor context, if any.
    fn optimistic_version_signal(&self) -> Signal<Option<u32>> {
        self.set_optimistic_version.into()
    }
}
