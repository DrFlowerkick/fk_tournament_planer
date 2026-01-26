//! Base parameters of a tournament

use crate::{
    Core, CoreError, CoreResult, CrMsg, CrTopic, SportError,
    utils::{
        id_version::IdVersion,
        normalize::normalize_ws,
        traits::ObjectIdVersion,
        validation::{FieldError, ValidationErrors, ValidationResult},
    },
};
use displaydoc::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// mode of tournament
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display)]
pub enum TournamentType {
    /// Scheduled Tournament
    #[default]
    Scheduled,
    /// Adhoc Tournament
    Adhoc,
}

impl From<String> for TournamentType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Scheduled" => TournamentType::Scheduled,
            "Adhoc" => TournamentType::Adhoc,
            _ => TournamentType::Scheduled, // default
        }
    }
}

/// mode of tournament
/// If there are Mode specific configuration values, which cannot be placed
/// in sub structures like Stage, Group, Match, etc., we may need to add them here.
/// For now, Swiss system needs number of rounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display)]
pub enum TournamentMode {
    /// Single Stage
    #[default]
    SingleStage,
    /// Pool and Final Stage
    PoolAndFinalStage,
    /// Two Pool Stages and Final Stage
    TwoPoolStagesAndFinalStage,
    /// Swiss System
    SwissSystem { num_rounds: u32 },
}

impl TournamentMode {
    pub fn get_num_of_stages(&self) -> u32 {
        match self {
            TournamentMode::SingleStage => 1,
            TournamentMode::PoolAndFinalStage => 2,
            TournamentMode::TwoPoolStagesAndFinalStage => 3,
            TournamentMode::SwissSystem { num_rounds: _ } => 1,
        }
    }
    pub fn get_stage_name(&self, stage_number: u32) -> Option<String> {
        match self {
            TournamentMode::SingleStage => Some("Single Stage".to_string()),
            TournamentMode::PoolAndFinalStage => match stage_number {
                0 => Some("Pool Stage".to_string()),
                1 => Some("Final Stage".to_string()),
                _ => None,
            },
            TournamentMode::TwoPoolStagesAndFinalStage => match stage_number {
                0 => Some("First Pool Stage".to_string()),
                1 => Some("Second Pool Stage".to_string()),
                2 => Some("Final Stage".to_string()),
                _ => None,
            },
            TournamentMode::SwissSystem { num_rounds: _ } => Some("Swiss System".to_string()),
        }
    }
}

/// status of tournament
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display)]
pub enum TournamentState {
    /// Draft
    #[default]
    Draft,
    /// Published
    Published,
    /// Running
    ActiveStage(u32),
    /// Finished
    Finished,
}

impl From<String> for TournamentState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Draft" => TournamentState::Draft,
            "Published" => TournamentState::Published,
            "Running" => TournamentState::ActiveStage(0),
            "Finished" => TournamentState::Finished,
            _ => TournamentState::Draft, // default
        }
    }
}

/// base parameters of a tournament
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TournamentBase {
    /// Unique identifier for the tournament
    id_version: IdVersion,
    /// name of tournament
    name: String,
    /// id of sport
    sport_id: Uuid,
    /// number of entrants; represents size of tournament
    num_entrants: u32,
    /// type of tournament
    t_type: TournamentType,
    /// mode of tournament
    mode: TournamentMode,
    /// state of tournament
    state: TournamentState,
}

impl ObjectIdVersion for TournamentBase {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

impl TournamentBase {
    /// Create a new `TournamentBase` with the given `IdVersion`.
    pub fn new(id_version: IdVersion) -> Self {
        TournamentBase {
            id_version,
            ..Default::default()
        }
    }

    /// Get the unique identifier of the sport configuration.
    pub fn get_id(&self) -> Uuid {
        self.id_version.get_id()
    }

    /// Get the version number of the sport configuration.
    pub fn get_version(&self) -> Option<u32> {
        self.id_version.get_version()
    }

    /// Get the name of the sport configuration.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the sport ID associated with this configuration.
    pub fn get_sport_id(&self) -> Uuid {
        self.sport_id
    }

    /// Get the number of entrants in the tournament.
    pub fn get_num_entrants(&self) -> u32 {
        self.num_entrants
    }

    /// Get the type of the tournament.
    pub fn get_tournament_type(&self) -> TournamentType {
        self.t_type
    }

    /// Get the mode of the tournament.
    pub fn get_tournament_mode(&self) -> TournamentMode {
        self.mode
    }

    /// Get the current state of the tournament.
    pub fn get_tournament_state(&self) -> TournamentState {
        self.state
    }

    /// Set the `IdVersion` of the sport configuration.
    pub fn set_id_version(&mut self, id_version: IdVersion) -> &mut Self {
        self.id_version = id_version;
        self
    }

    /// Set the name of the tournament with normalization
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::TournamentBase;
    ///
    /// // Start from default.
    /// let mut tournament = TournamentBase::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// tournament.set_name("  Fun   Sport  Tournament  ".to_string());
    /// assert_eq!(tournament.get_name(), "Fun Sport Tournament");
    /// ```
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = normalize_ws(name.into());
        self
    }

    /// Set the sport ID associated with this configuration.
    pub fn set_sport_id(&mut self, sport_id: Uuid) -> &mut Self {
        self.sport_id = sport_id;
        self
    }

    /// Set the number of entrants in the tournament.
    pub fn set_num_entrants(&mut self, num_entrants: u32) -> &mut Self {
        self.num_entrants = num_entrants;
        self
    }

    /// Set the type of the tournament.
    pub fn set_tournament_type(&mut self, t_type: TournamentType) -> &mut Self {
        self.t_type = t_type;
        self
    }

    /// Set the mode of the tournament.
    pub fn set_tournament_mode(&mut self, mode: TournamentMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set the current state of the tournament.
    pub fn set_tournament_state(&mut self, state: TournamentState) -> &mut Self {
        self.state = state;
        self
    }

    /// Validate the tournament configuration.
    pub fn validate(&self) -> ValidationResult<()> {
        let mut errs = ValidationErrors::new();
        let object_id = self.get_id();

        if self.name.trim().is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("name"))
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }

        if self.num_entrants < 2 {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("num_entrants"))
                    .add_message("number of entrants must be at least 2")
                    .set_object_id(object_id)
                    .build(),
            );
        }

        match self.mode {
            TournamentMode::SwissSystem { num_rounds } => {
                if num_rounds == 0 {
                    errs.add(
                        FieldError::builder()
                            .set_field(String::from("mode.num_rounds"))
                            .add_message("number of rounds must be > 0")
                            .set_object_id(object_id)
                            .build(),
                    );
                }
            }
            _ => {}
        }

        // ToDo: refine validation of active stage based on mode, when active stage is implemented
        let max_num_stages = match self.mode {
            // in Swiss System, each round is a stage
            TournamentMode::SwissSystem { num_rounds } => num_rounds,
            TournamentMode::SingleStage => 1,
            TournamentMode::PoolAndFinalStage => 2,
            TournamentMode::TwoPoolStagesAndFinalStage => 3,
        };

        if let TournamentState::ActiveStage(active_stage) = self.state {
            // index stages from 0
            if active_stage >= max_num_stages {
                errs.add(
                    FieldError::builder()
                        .set_field(String::from("state"))
                        .add_message(
                            "active stage exceeds maximum number of stages for the tournament mode",
                        )
                        .set_object_id(object_id)
                        .build(),
                );
            }
        }

        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(())
    }
}

pub struct TournamentBaseState {
    tournament: TournamentBase,
}

// switch state to sport config state
impl<S> Core<S> {
    pub fn as_tournament_base_state(&self) -> Core<TournamentBaseState> {
        self.switch_state(TournamentBaseState {
            tournament: TournamentBase::default(),
        })
    }
}

impl Core<TournamentBaseState> {
    pub fn get(&self) -> &TournamentBase {
        &self.state.tournament
    }
    pub fn get_mut(&mut self) -> &mut TournamentBase {
        &mut self.state.tournament
    }
    fn validate(&self, tournament: &TournamentBase) -> CoreResult<()> {
        if self.sport_plugins.get(&tournament.sport_id).is_none() {
            return Err(CoreError::from(SportError::UnknownSportId(
                tournament.sport_id,
            )));
        };
        tournament.validate().map_err(CoreError::from)?;
        Ok(())
    }
    pub async fn load(&mut self, id: Uuid) -> CoreResult<Option<&TournamentBase>> {
        if let Some(tournament) = self.database.get_tournament_base(id).await? {
            self.state.tournament = tournament;
            self.validate(&self.state.tournament)?;

            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }
    pub async fn save(&mut self) -> CoreResult<&TournamentBase> {
        self.validate(&self.state.tournament)?;
        self.state.tournament = self
            .database
            .save_tournament_base(&self.state.tournament)
            .await?;

        // publish change of tournament base to client registry
        let id = self.state.tournament.get_id();
        let version =
            self.state.tournament.get_version().expect(
                "expecting save_tournament_base to return always an existing id and version",
            );
        let notice = CrTopic::TournamentBase(id);
        let msg = CrMsg::TournamentBaseUpdated { id, version };
        self.client_registry.publish(notice, msg).await?;
        Ok(self.get())
    }
    pub async fn list_sport_tournaments(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<TournamentBase>> {
        let tournaments = self
            .database
            .list_tournament_bases(sport_id, name_filter, limit)
            .await?;

        for tournament in &tournaments {
            self.validate(tournament)?;
        }
        Ok(tournaments)
    }
}
