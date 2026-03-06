//! stage of a tournament

use super::base::{TournamentBase, TournamentMode};
use crate::{
    Core, CoreError, CoreResult, CrMsg, CrTopic,
    utils::{
        id_version::IdVersion,
        traits::{ObjectIdVersion, ObjectNumber},
        validation::{FieldError, ValidationErrors, ValidationResult},
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// stage of a tournament
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stage {
    /// id and version of stage in tournament
    id_version: IdVersion,
    /// id of tournament
    tournament_id: Uuid,
    /// scheduled stage number in tournament
    number: u32,
    /// number of entrants per group.
    /// The length of this vector defines the number of groups.
    /// The sum of values must match the total number of entrants (or survivors from prev stage).
    group_sizes: Vec<u32>,
}

impl Default for Stage {
    fn default() -> Self {
        Stage {
            id_version: IdVersion::default(),
            tournament_id: Uuid::nil(),
            number: 0,
            // Default is one group (e.g. for Swiss System or final)
            // Ideally should be initialized based on tournament entrants when created properly
            group_sizes: vec![0],
        }
    }
}

impl ObjectIdVersion for Stage {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

impl ObjectNumber for Stage {
    fn get_object_number(&self) -> u32 {
        self.number
    }
}

impl Stage {
    /// Create a new `Stage` with the given `IdVersion`.
    pub fn new(id_version: IdVersion) -> Self {
        Stage {
            id_version,
            ..Default::default()
        }
    }

    /// Get the unique identifier of the stage.
    pub fn get_id(&self) -> Uuid {
        self.id_version.get_id()
    }

    /// Get the version number of the stage.
    pub fn get_version(&self) -> Option<u32> {
        self.id_version.get_version()
    }

    /// Returns the tournament ID.
    pub fn get_tournament_id(&self) -> Uuid {
        self.tournament_id
    }

    /// Get the scheduled stage number in tournament.
    pub fn get_number(&self) -> u32 {
        self.number
    }

    /// Get the number of groups in stage.
    /// Calculated field based on group_sizes configuration.
    pub fn get_num_groups(&self) -> u32 {
        self.group_sizes.len() as u32
    }

    /// Get the configured sizes for each group
    pub fn get_group_sizes(&self) -> &[u32] {
        &self.group_sizes
    }

    /// Set the `IdVersion` of the stage.
    pub fn set_id_version(&mut self, id_version: IdVersion) -> &mut Self {
        self.id_version = id_version;
        self
    }

    /// Set the tournament ID.
    pub fn set_tournament_id(&mut self, tournament_id: Uuid) -> &mut Self {
        self.tournament_id = tournament_id;
        self
    }

    /// Set the scheduled stage number in tournament.
    pub fn set_number(&mut self, number: u32) -> &mut Self {
        self.number = number;
        self
    }

    /// Set the number of groups in stage.
    pub fn set_number_of_groups(&mut self, num_groups: u32) -> &mut Self {
        match self.get_num_groups().cmp(&num_groups) {
            std::cmp::Ordering::Less => {
                // Add new groups with default size 0
                self.group_sizes.extend(
                    std::iter::repeat(0).take((num_groups - self.get_num_groups()) as usize),
                );
            }
            std::cmp::Ordering::Greater => {
                // Remove extra groups from the end
                self.group_sizes.truncate(num_groups as usize);
            }
            std::cmp::Ordering::Equal => {
                // No change needed
            }
        }
        self
    }

    /// Set the group sizes configuration directly.
    pub fn set_group_sizes(&mut self, sizes: Vec<u32>) -> &mut Self {
        self.group_sizes = sizes;
        self
    }

    /// Set the size of a specific group by index.
    pub fn set_group_size(&mut self, group_index: usize, size: u32) -> &mut Self {
        if let Some(group_size) = self.group_sizes.get_mut(group_index) {
            *group_size = size;
        }
        self
    }

    /// Helper to distribute entrants evenly across N groups.
    /// Remaining entrants are distributed one by one to the first groups.
    /// E.g. 10 entrants, 3 groups -> [4, 3, 3]
    pub fn distribute_groups_evenly(
        &mut self,
        num_total_entrants: u32,
        num_groups: u32,
    ) -> &mut Self {
        if num_groups == 0 {
            self.group_sizes = vec![];
            return self;
        }

        let base_size = num_total_entrants / num_groups;
        let remainder = num_total_entrants % num_groups;

        let mut sizes = Vec::with_capacity(num_groups as usize);
        for i in 0..num_groups {
            let size = base_size + if i < remainder { 1 } else { 0 };
            sizes.push(size);
        }
        self.group_sizes = sizes;
        self
    }

    /// Validate the stage configuration based on the provided tournament settings.
    pub fn validate(&self, tournament: &TournamentBase) -> ValidationResult<()> {
        let mut errs = ValidationErrors::new();
        let object_id = self.get_id();

        // Check if stage belongs to the correct tournament
        if tournament.get_id() != self.tournament_id {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("tournament_id"))
                    .add_message("Stage tournament ID does not match the provided tournament")
                    .set_object_id(object_id)
                    .build(),
            );
        }

        // Validate stage number against max stages
        let max_stages = tournament.get_tournament_mode().get_num_of_stages();
        if self.number >= max_stages {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("number"))
                    .add_message(format!(
                        "Stage number {} exceeds maximum allowed stages ({}) for mode {}",
                        self.number,
                        max_stages,
                        tournament.get_tournament_mode()
                    ))
                    .set_object_id(object_id)
                    .build(),
            );
        }

        // Validate number of groups (implicitly validates group_sizes vector)
        let num_groups = self.get_num_groups();

        if num_groups == 0 {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("group_sizes"))
                    .add_message("Number of groups must be at least 1")
                    .set_object_id(object_id)
                    .build(),
            );
        }

        if num_groups > tournament.get_num_entrants() / 2 {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("num_groups"))
                    .add_message("Number of groups cannot exceed half the number of entrants")
                    .set_object_id(object_id)
                    .build(),
            );
        }

        // Validate total entrants sum
        let configured_entrants: u32 = self.group_sizes.iter().sum();
        let expected_entrants = tournament.get_num_entrants();
        // Note: For later stages, this logic might need adjustment if entrants drop out,
        // but for stage definition, the sum usually acts as a sanity check against the count
        // expected in this stage. Assuming for now all entrants are part of the stage structure.
        if configured_entrants != expected_entrants {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("group_sizes"))
                    .add_message(format!(
                        "Sum of group sizes ({}) must match total tournament entrants ({})",
                        configured_entrants, expected_entrants
                    ))
                    .set_object_id(object_id)
                    .build(),
            );
        }

        // Mode specific validation
        let mode = tournament.get_tournament_mode();

        // Validate number of groups against mode constraints
        if matches!(mode, TournamentMode::SingleStage) {
            if num_groups != 1 {
                errs.add(
                    FieldError::builder()
                        .set_field(String::from("num_groups"))
                        .add_message(
                            "Single Stage mode must have exactly 1 group (the whole field)",
                        )
                        .set_object_id(object_id)
                        .build(),
                );
            }
        }

        // Specific constraint: Swiss System has 1 group in stage (the whole field)
        if let TournamentMode::SwissSystem { .. } = mode {
            if num_groups > 1 {
                errs.add(
                    FieldError::builder()
                        .set_field(String::from("num_groups"))
                        .add_message("Swiss System has 1 group in stage (the whole field)")
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

pub struct StageState {
    tournament_id: Uuid,
    tournament: Option<TournamentBase>,
    stage: Stage,
}

// switch state to sport config state
impl<S> Core<S> {
    pub fn as_stage_state(&self, tournament_id: Uuid) -> Core<StageState> {
        self.switch_state(StageState {
            tournament_id,
            tournament: None,
            stage: Stage::default(),
        })
    }
}

impl Core<StageState> {
    pub fn get(&self) -> &Stage {
        &self.state.stage
    }
    pub fn get_mut(&mut self) -> &mut Stage {
        &mut self.state.stage
    }
    pub fn get_tournament(&self) -> Option<&TournamentBase> {
        self.state.tournament.as_ref()
    }
    async fn try_load_tournament(&mut self) -> CoreResult<()> {
        if self.state.tournament.is_none() {
            if let Some(tournament) = self
                .as_tournament_base_state()
                .load(self.state.tournament_id)
                .await?
            {
                self.state.tournament = Some(tournament.clone());
            }
        }
        Ok(())
    }
    fn validate(&self) -> CoreResult<()> {
        if let Some(tournament) = self.state.tournament.as_ref() {
            self.state
                .stage
                .validate(tournament)
                .map_err(CoreError::from)?;
        }
        Ok(())
    }
    pub async fn load_by_id(&mut self, id: Uuid) -> CoreResult<Option<&Stage>> {
        if let Some(stage) = self.database.get_stage_by_id(id).await? {
            self.state.stage = stage;
            self.try_load_tournament().await?;
            self.validate()?;
            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }
    pub async fn load_by_number(&mut self, number: u32) -> CoreResult<Option<&Stage>> {
        self.try_load_tournament().await?;
        if let Some(stage) = self
            .database
            .get_stage_by_number(self.state.tournament_id, number)
            .await?
        {
            self.state.stage = stage;
            self.try_load_tournament().await?;
            self.validate()?;
            return Ok(Some(self.get()));
        }
        Ok(None)
    }
    pub async fn save(&mut self) -> CoreResult<&Stage> {
        // Validation of stage requires valid tournament base.
        // When saving a new tournament, the objects should be saved in order:
        // TournamentBase, Stages, Groups, Rounds, Matches
        // Otherwise saved objects may not be validated before saving.
        self.try_load_tournament().await?;
        self.validate()?;
        self.state.stage = self.database.save_stage(&self.state.stage).await?;

        // publish change of stage to client registry
        let id = self.state.stage.get_id();
        let version = self
            .state
            .stage
            .get_version()
            .expect("expecting save_stage to return always an existing id and version");
        let notice = if version == 0 {
            CrTopic::NewStage {
                tournament_base_id: self.state.tournament_id,
            }
        } else {
            CrTopic::Stage { stage_id: id }
        };
        let msg = CrMsg::StageUpdated { id, version };
        self.client_registry.publish(notice, msg).await?;
        Ok(self.get())
    }
    pub async fn list_stage_ids_of_tournament(&mut self) -> CoreResult<Vec<(Uuid, u32)>> {
        self.try_load_tournament().await?;
        if let Some(tournament) = self.state.tournament.as_ref() {
            let stages = self
                .database
                .list_stage_ids_of_tournament(
                    self.state.tournament_id,
                    tournament.get_tournament_mode().get_num_of_stages(),
                )
                .await?;
            Ok(stages)
        } else {
            Ok(vec![])
        }
    }
}
