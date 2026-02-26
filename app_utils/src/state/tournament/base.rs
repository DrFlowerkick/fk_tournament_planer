//! base editor context

use crate::{
    error::{
        AppError, AppResult, map_db_unique_violation_to_field_error, strategy::handle_write_error,
    },
    params::{ParamQuery, SportIdQuery},
    server_fn::tournament_base::{SaveTournamentBase, load_tournament_base},
    state::{
        EditorContext, EditorContextWithResource, EditorOptions, activity_tracker::ActivityTracker,
        toast_state::ToastContext,
    },
};
use app_core::{
    CrTopic, Tournament, TournamentBase, TournamentMode, TournamentState, TournamentType,
    utils::{
        id_version::IdVersion,
        validation::{FieldError, ValidationResult},
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::prelude::*;
use uuid::Uuid;

pub struct BaseEditorContextOptions {
    pub object_id: Option<Uuid>,
    pub local_tournament: RwSignal<Option<Tournament>>,
    pub origin_tournament: RwSignal<Option<Tournament>>,
}

impl EditorOptions for BaseEditorContextOptions {
    fn object_id(&self) -> Option<Uuid> {
        self.object_id
    }
}

#[derive(Clone, Copy)]
pub struct BaseEditorContext {
    // --- state & derived signals ---
    /// The local editable tournament base in the editor context.
    pub local: Signal<Option<TournamentBase>>,
    /// SignalSetter for setting the local tournament base in the editor context.
    set_local: SignalSetter<Option<TournamentBase>>,
    /// The original tournament base loaded from storage.
    origin: Signal<Option<TournamentBase>>,
    /// SignalSetter for setting the original tournament base in the editor context.
    set_origin: SignalSetter<Option<TournamentBase>>,
    /// Read slice for accessing the validation result of the tournament base
    pub validation_result: Signal<ValidationResult<()>>,
    /// WriteSignal for setting a unique violation error on the name field, if any
    pub set_unique_violation_error: WriteSignal<Option<FieldError>>,
    /// Read slice for checking if the tournament base is in a state where editing is disabled
    /// (e.g. when tournament is active or finished)
    pub is_disabled_base_editing: Signal<bool>,
    /// Read slice for checking if the stage editor should be skipped based on the tournament mode
    pub skip_stage_editor: Signal<bool>,

    // --- Signals, Slices & Callbacks for form fields ---
    /// Signal slice for the id field
    pub id: Signal<Option<Uuid>>,
    /// Signal slice for the version field
    pub version: Signal<Option<u32>>,
    /// Signal slice for the name field
    pub name: Signal<Option<String>>,
    /// Callback for updating the name field
    pub set_name: Callback<Option<String>>,
    /// Read slice for accessing the tournament base number of entrants, if any
    pub num_entrants: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of entrants
    pub set_num_entrants: Callback<Option<u32>>,
    /// Read slice for accessing the tournament base type, if any
    pub tournament_type: Signal<Option<TournamentType>>,
    /// Read slice for accessing the tournament base mode, if any
    pub mode: Signal<Option<TournamentMode>>,
    /// Write slice for setting the tournament base mode
    pub set_mode: Callback<Option<TournamentMode>>,
    /// Read slice for accessing the tournament base number of rounds for Swiss System, if any
    pub num_rounds_swiss_system: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of rounds for Swiss System
    pub set_num_rounds_swiss_system: Callback<Option<u32>>,
    /// Read slice for accessing the tournament state, if any
    pub tournament_state: Signal<Option<TournamentState>>,

    // --- Resource & server action state ---
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    pub set_optimistic_version: RwSignal<Option<u32>>,
    /// WriteSignal for setting the resource id in the editor context, which triggers loading of the tournament base resource
    set_resource_id: WriteSignal<Option<Uuid>>,
    /// Resource for loading the tournament base based on the given id in the editor options
    pub load_tournament_base: LocalResource<AppResult<Option<TournamentBase>>>,
    /// Server action for saving the tournament base based on the current state of the editor context
    pub save_tournament_base: ServerAction<SaveTournamentBase>,
    /// Callback after successful save to e.g. navigate to the new tournament base or show a success toast.
    pub post_save_callback: StoredValue<Option<Callback<TournamentBase>>>,
}

impl EditorContext for BaseEditorContext {
    type ObjectType = TournamentBase;
    type NewEditorOptions = BaseEditorContextOptions;

    fn new(options: Self::NewEditorOptions) -> Self {
        // ---- global state & context ----
        let toast_ctx = expect_context::<ToastContext>();
        let activity_tracker = expect_context::<ActivityTracker>();
        let component_id = StoredValue::new(Uuid::new_v4());
        // remove errors on unmount
        on_cleanup(move || {
            activity_tracker.remove_component(component_id.get_value());
        });

        // ---- signals & slices ----
        let (local, set_local) = create_slice(
            options.local_tournament,
            |local_tournament| local_tournament.as_ref().map(|t| t.get_base().clone()),
            |local_tournament, new_base: Option<TournamentBase>| {
                if let Some(new_base) = new_base {
                    if let Some(t) = local_tournament {
                        t.set_base(new_base);
                    } else {
                        let mut new_tournament = Tournament::new();
                        new_tournament.set_base(new_base);
                        *local_tournament = Some(new_tournament);
                    }
                } else {
                    *local_tournament = None;
                }
            },
        );
        let (origin, set_origin) = create_slice(
            options.origin_tournament,
            |origin_tournament| origin_tournament.as_ref().map(|t| t.get_base().clone()),
            |origin_tournament, new_base: Option<TournamentBase>| {
                if let Some(new_base) = new_base {
                    if let Some(origin) = origin_tournament {
                        origin.set_base(new_base);
                    } else {
                        let mut new_tournament = Tournament::new();
                        new_tournament.set_base(new_base);
                        *origin_tournament = Some(new_tournament);
                    }
                } else {
                    *origin_tournament = None;
                }
            },
        );
        let (unique_violation_error, set_unique_violation_error) = signal(None::<FieldError>);
        let validation_result = Signal::derive(move || {
            let vr = local.with(|local| {
                if let Some(base) = local {
                    base.validate()
                } else {
                    ValidationResult::Ok(())
                }
            });
            if let Some(unique_err) = unique_violation_error.get() {
                if let Err(mut validation_errors) = vr {
                    validation_errors.add(unique_err);
                    Err(validation_errors)
                } else {
                    Err(unique_err.into())
                }
            } else {
                vr
            }
        });

        let id = create_read_slice(options.local_tournament, |local_tournament| {
            local_tournament.as_ref().map(|t| t.get_base().get_id())
        });
        let version = create_read_slice(options.local_tournament, move |local_tournament| {
            local_tournament
                .as_ref()
                .and_then(|t| t.get_base().get_version())
        });

        let (name, set_name) = create_slice(
            options.local_tournament,
            |local_tournament| {
                local_tournament
                    .as_ref()
                    .map(|t| t.get_base().get_name().to_string())
            },
            move |local_tournament, name: String| {
                if let Some(t) = local_tournament {
                    t.set_base_name(name);
                    // Clear unique violation error on name change, if any
                    set_unique_violation_error.set(None);
                }
            },
        );
        let set_name = Callback::new(move |name: Option<String>| {
            set_name.set(name.unwrap_or_default());
        });
        let (num_entrants, set_num_entrants) = create_slice(
            options.local_tournament,
            |local_tournament| {
                local_tournament
                    .as_ref()
                    .map(|t| t.get_base().get_num_entrants())
            },
            |local_tournament, num_entrants: u32| {
                if let Some(t) = local_tournament {
                    t.set_base_num_entrants(num_entrants);
                }
            },
        );
        let set_num_entrants = Callback::new(move |num_entrants: Option<u32>| {
            set_num_entrants.set(num_entrants.unwrap_or_default());
        });
        let tournament_type = create_read_slice(options.local_tournament, |local_tournament| {
            local_tournament
                .as_ref()
                .map(|t| t.get_base().get_tournament_type())
        });
        let (mode, set_mode) = create_slice(
            options.local_tournament,
            |local_tournament| {
                local_tournament
                    .as_ref()
                    .map(|t| t.get_base().get_tournament_mode())
            },
            |local_tournament, mode: TournamentMode| {
                if let Some(t) = local_tournament {
                    t.set_base_mode(mode);
                }
            },
        );
        let set_mode = Callback::new(move |mode: Option<TournamentMode>| {
            if let Some(mode) = mode {
                set_mode.set(mode);
            }
        });
        let skip_stage_editor = Signal::derive(move || {
            matches!(
                mode.get(),
                Some(TournamentMode::SingleStage) | Some(TournamentMode::SwissSystem { .. })
            )
        });
        let (num_rounds_swiss_system, set_num_rounds_swiss_system) = create_slice(
            options.local_tournament,
            |local_tournament| {
                local_tournament
                    .as_ref()
                    .and_then(|t| t.get_base().get_num_rounds_swiss_system())
            },
            |local_tournament, num_rounds_swiss: u32| {
                if let Some(t) = local_tournament {
                    t.set_base_num_rounds_swiss_system(num_rounds_swiss);
                }
            },
        );
        let set_num_rounds_swiss_system = Callback::new(move |num_rounds_swiss: Option<u32>| {
            set_num_rounds_swiss_system.set(num_rounds_swiss.unwrap_or_default());
        });
        let tournament_state = create_read_slice(options.local_tournament, |local_tournament| {
            local_tournament
                .as_ref()
                .map(|t| t.get_base().get_tournament_state())
        });
        let is_disabled_base_editing = Signal::derive(move || {
            matches!(
                tournament_state.get(),
                Some(TournamentState::ActiveStage(_)) | Some(TournamentState::Finished)
            )
        });

        // ---- tournament base resource ----
        let (resource_id, set_resource_id) = signal(options.object_id);
        let set_optimistic_version = RwSignal::new(None::<u32>);

        // resource to load tournament base
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
        let load_tournament_base = LocalResource::new(move || async move {
            if let Some(id) = resource_id.get() {
                match activity_tracker
                    .track_activity_wrapper(component_id.get_value(), load_tournament_base(id))
                    .await
                {
                    Ok(None) => Err(AppError::ResourceNotFound(
                        "Tournament Base".to_string(),
                        id,
                    )),
                    res => res,
                }
            } else {
                Ok(None)
            }
        });

        let topic = Signal::derive(move || resource_id.get().map(|id| CrTopic::Address(id)));
        let refetch = Callback::new(move |()| {
            load_tournament_base.refetch();
        });
        use_client_registry_socket(topic, set_optimistic_version.into(), refetch);

        // ---- tournament base server action ----
        let save_tournament_base = ServerAction::<SaveTournamentBase>::new();
        let save_tournament_base_pending = save_tournament_base.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), save_tournament_base_pending);

        let post_save_callback = StoredValue::new(None::<Callback<TournamentBase>>);

        // handle save result
        Effect::new(move || {
            if let Some(stb_result) = save_tournament_base.value().get() {
                save_tournament_base.clear();
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
                        // transform unique violation error into Validation Error for name, if any
                        if let Some(object_id) = id.get()
                            && let Some(field_error) =
                                map_db_unique_violation_to_field_error(&err, object_id, "name")
                        {
                            set_unique_violation_error.set(Some(field_error));
                        } else {
                            handle_write_error(&toast_ctx, &err);
                        }
                    }
                }
            }
        });

        BaseEditorContext {
            local,
            set_local,
            origin,
            set_origin,
            validation_result,
            set_unique_violation_error,
            is_disabled_base_editing,
            id,
            version,
            name,
            set_name,
            num_entrants,
            set_num_entrants,
            tournament_type,
            mode,
            set_mode,
            skip_stage_editor,
            num_rounds_swiss_system,
            set_num_rounds_swiss_system,
            tournament_state,
            set_optimistic_version,
            set_resource_id,
            load_tournament_base,
            save_tournament_base,
            post_save_callback,
        }
    }

    /// Get the original tournament base currently loaded in the editor context, if any.
    fn origin_signal(&self) -> Signal<Option<Self::ObjectType>> {
        self.origin.into()
    }

    /// Set an existing tournament base in the editor context.
    fn set_object(&self, base: Self::ObjectType) {
        let id = base.get_id();
        self.set_local.set(Some(base.clone()));
        self.set_optimistic_version.set(base.get_version());
        self.set_origin.set(Some(base));
        self.set_resource_id.set(Some(id));
    }

    /// Create a new tournament base in the editor context with a new UUID and default values.
    fn new_object(&self) -> Option<Uuid> {
        let sport_id = SportIdQuery::use_param_query();
        if let Some(sport_id) = sport_id.get() {
            let mut base = TournamentBase::default();
            base.set_sport_id(sport_id);
            let id = base.get_id();

            self.set_resource_id.set(None);
            self.set_local.set(Some(base));
            self.set_optimistic_version.set(None);
            self.set_origin.set(None);
            Some(id)
        } else {
            None
        }
    }
}

impl EditorContextWithResource for BaseEditorContext {
    /// Create a new object from a given tournament base by copying it and assigning a new UUID, then set it in the editor context.
    fn copy_object(&self, mut base: Self::ObjectType) -> Option<Uuid> {
        let id = Uuid::new_v4();
        base.set_id_version(IdVersion::new(id, None)).set_name("");

        self.set_resource_id.set(None);
        self.set_local.set(Some(base));
        self.set_optimistic_version.set(None);
        self.set_origin.set(None);
        Some(id)
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
