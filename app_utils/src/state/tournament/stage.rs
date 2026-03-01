//! stage editor context

use crate::{
    error::{AppError, ComponentError, ComponentResult, strategy::handle_write_error},
    server_fn::stage::{SaveStage, load_stage_by_id},
    state::{
        EditorContext, EditorContextWithResource, EditorOptions, activity_tracker::ActivityTracker,
        error_state::PageErrorContext, toast_state::ToastContext,
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
    pub origin_tournament: RwSignal<Option<Tournament>>,
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
    /// The original stage loaded from storage.
    origin: Signal<Option<Stage>>,
    /// SignalSetter for setting the original stage in the editor context.
    set_origin: SignalSetter<Option<Stage>>,
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

    // --- Resource & server action state ---
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    set_optimistic_version: RwSignal<Option<u32>>,
    /// Resource for loading the stage based on the given id in the editor options
    pub load_stage: LocalResource<ComponentResult<Option<Stage>>>,
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
        let (origin, set_origin) = create_slice(
            options.origin_tournament,
            move |origin_tournament| {
                id.get().and_then(|id| {
                    origin_tournament
                        .as_ref()
                        .and_then(|t| t.get_stage_by_id(id).cloned())
                })
            },
            |origin_tournament, new_stage: Option<Stage>| {
                if let Some(new_stage) = new_stage
                    && let Some(origin) = origin_tournament
                {
                    origin.set_stage(new_stage);
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
                        .map(|s| s.get_num_groups())
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

        // ---- tournament stage resource ----
        let (resource_id, set_resource_id) = signal(options.object_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // resource to load tournament stage
        /*let load_tournament_base = Resource::new(
            move || resource_id.get(),
            move |maybe_id| async move {
                if let Some(id) = maybe_id {
                    match activity_tracker
                        .track_activity_wrapper(component_id.get_value(), load_tournament_base(id))
                        .await
                    {
                        Ok(None) => {
                            Err(AppError::ResourceNotFound("Tournament Base".to_string(), id))
                        }
                        res => res,
                    }
                } else {
                    Ok(None)
                }
            },
        );*/
        // At current state of leptos SSR does not provide stable rendering (meaning during initial load Hydration
        // errors occur until the page is fully rendered and the app "transformed" into a SPA). For this reason
        // we use a LocalResource here, which does not cause hydration errors.
        // ToDo: investigate how to use Resource without hydration errors, since Resource provides better
        // ergonomics for loading states and error handling.
        let load_stage = LocalResource::new(move || async move {
            if let Some(id) = resource_id.get()
                && let Some(tournament_id) = tournament_id.get()
            {
                match activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        load_stage_by_id(tournament_id, id),
                    )
                    .await
                {
                    Ok(None) => Err(AppError::ResourceNotFound(
                        "Tournament Stage".to_string(),
                        id,
                    )),
                    res => res,
                }
            } else {
                Ok(None)
            }
            .map_err(|app_error| ComponentError::new(component_id.get_value(), app_error))
        });
        let refetch = Callback::new(move |()| {
            load_stage.refetch();
        });
        page_err_ctx.register_retry_handler(component_id.get_value(), refetch);

        let topic = Signal::derive(move || resource_id.get().map(|id| CrTopic::Address(id)));
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // ---- tournament stage server action ----
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
                        set_origin.set(Some(tb.clone()));

                        if let Some(callback) = post_save_callback.get_value() {
                            callback.run(tb);
                        }
                    }
                    Err(err) => {
                        // version reset for parallel editing
                        set_optimistic_version.set(version.get());
                        handle_write_error(&toast_ctx, &err);
                    }
                }
            }
        });

        StageEditorContext {
            stage_number: options.stage_number,
            local,
            set_local,
            origin,
            set_origin,
            validation_result,
            is_disabled_stage_editing,
            id,
            version,
            tournament_id,
            number,
            num_groups,
            set_num_groups,
            set_optimistic_version,
            load_stage,
            save_stage,
            post_save_callback,
        }
    }

    /// Get the original tournament stage currently loaded in the editor context, if any.
    fn origin_signal(&self) -> Signal<Option<Self::ObjectType>> {
        self.origin.into()
    }

    /// Set an existing tournament stage in the editor context.
    fn set_object(&self, stage: Self::ObjectType) {
        self.set_local.set(Some(stage.clone()));
        self.set_optimistic_version.set(stage.get_version());
        self.set_origin.set(Some(stage));
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
            self.set_origin.set(None);
            Some(id)
        } else {
            None
        }
    }
}

impl EditorContextWithResource for StageEditorContext {
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
            self.set_origin.set(None);
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
