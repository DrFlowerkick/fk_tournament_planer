/// Tournament functionality
///
/// For detailed domain knowledge, rules, and architectural decisions,
/// please refer to `.github/instructions/domain_knowledge.instructions.md`.
pub mod base;
pub mod stage;

pub use base::*;
pub use stage::*;

use crate::{
    Group,
    utils::{
        id_version::IdVersion,
        traits::{Diffable, ObjectIdVersion, ObjectNumber},
        validation::{ValidationErrors, ValidationResult},
    },
};
use petgraph::{
    Direction,
    graphmap::DiGraphMap,
    visit::{Bfs, Walker},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DependencyType {
    Stage,
    Group,
}

/// Tournament holds all data of a tournament.
///
/// See `.github/instructions/domain_knowledge.instructions.md` for architectural details regarding
/// graph structure, orphan handling, and bottom-up heritage.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tournament {
    /// base of tournament
    pub base: TournamentBase,
    /// map of tournament dependencies
    pub structure: DiGraphMap<Uuid, DependencyType>,
    /// stages associated with the tournament
    pub stages: HashMap<Uuid, Stage>,
    /// groups associated with stages (not yet used)
    pub groups: HashMap<Uuid, Group>,
}

// tournament id and version are determined by the tournament base.
impl ObjectIdVersion for Tournament {
    fn get_id_version(&self) -> IdVersion {
        self.base.get_id_version()
    }
}

impl Tournament {
    pub fn new() -> Self {
        Tournament {
            base: TournamentBase::default(),
            structure: DiGraphMap::new(),
            stages: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    // --- Generators for new Tournament Objects ---

    /// Creates a new tournament base with a new ID and default settings.
    pub fn new_base(&mut self, sport_id: Uuid) {
        let mut base = TournamentBase::default();
        base.set_sport_id(sport_id);
        self.set_base(base);
    }

    /// Creates a new stage with a new ID and adds it to the tournament.
    /// If a stage with the same number already exists, it is not added.
    /// This  reduces the risk of having multiple stages with same number.
    /// Returns false if stage was added, true otherwise.
    pub fn new_stage(&mut self, stage_number: u32) -> bool {
        if self.get_stage_by_number(stage_number).is_some() {
            // stage with this number already exists
            return true;
        }
        let tournament_id = self.base.get_id();
        let mut stage = Stage::default();
        // set required fields
        stage
            .set_number(stage_number)
            .set_tournament_id(tournament_id);
        self.set_stage(stage)
    }

    // ---- Setters for Tournament Objects after loading from database ----

    /// Setter for the tournament base.
    pub fn set_base(&mut self, base: TournamentBase) {
        let new_id = base.get_id();

        // Ensure Graph Node exists
        self.structure.add_node(new_id);

        // Set base
        self.base = base;

        // Validation: Check if changes invalidate child objects (e.g. Mode change -> fewer stages)
        self.unlink_excess_stages();
    }

    /// Sets the name of the tournament base.
    pub fn set_base_name(&mut self, name: impl Into<String>) {
        self.base.set_name(name);
    }

    /// Sets the number of entrants of the tournament base.
    pub fn set_base_num_entrants(&mut self, num_entrants: u32) {
        self.base.set_num_entrants(num_entrants);
    }

    /// Sets the tournament mode of the tournament base.
    pub fn set_base_mode(&mut self, mode: TournamentMode) {
        self.base.set_tournament_mode(mode);

        // Validation: Check if changes invalidate child objects (e.g. Mode change -> fewer stages)
        self.unlink_excess_stages();
    }

    pub fn set_base_num_rounds_swiss_system(&mut self, num_rounds_swiss: u32) {
        if matches!(
            self.base.get_tournament_mode(),
            TournamentMode::SwissSystem { .. }
        ) {
            self.base.set_num_rounds_swiss_system(num_rounds_swiss);
        }
    }

    /// Sets a stage to the state and links it to the tournament.
    /// If a stage with the same number but different ID already exists,
    /// it is not replaced and new stage is not added.
    /// Returns false if set was successful, true otherwise.
    pub fn set_stage(&mut self, stage: Stage) -> bool {
        // check for existing stage with same number but different ID
        let stage_id = stage.get_id();
        if let Some(stage) = self.get_stage_by_number(stage.get_number())
            && stage.get_id() != stage_id
        {
            // ToDo: handle this case better by e.g. showing an error message to user, instead of silently ignoring the new stage
            return true;
        }

        // Link to tournament root, which although adds the node if missing
        let base_id = self.base.get_id();
        self.structure
            .add_edge(base_id, stage_id, DependencyType::Stage);

        // Add to stages map
        self.stages.insert(stage_id, stage);

        // Validation: Check if changes invalidate child objects (e.g. fewer groups)
        self.unlink_excess_groups(stage_id);
        false
    }

    /// Clears a stage from the state.
    /// Returns the removed stage if it existed.
    pub fn clear_stage(&mut self, stage_id: Uuid) -> Option<Stage> {
        // Remove node of stage
        self.structure.remove_node(stage_id);
        // Remove stage from stages map
        self.stages.remove(&stage_id)
    }

    pub fn set_stage_number_of_groups(&mut self, stage_id: Uuid, num_groups: u32) -> bool {
        let Some(stage) = self.stages.get_mut(&stage_id) else {
            return true;
        };

        // We set the number of groups in the stage, which may require invalidating excess groups if the number is reduced.
        stage.set_number_of_groups(num_groups);

        // Validation: Check if changes invalidate child objects (e.g. fewer groups)
        self.unlink_excess_groups(stage_id);
        false
    }

    /// Sets the group sizes for a stage.
    /// Returns true if stage is not present.
    pub fn set_stage_group_size(&mut self, stage_id: Uuid, group_index: usize, size: u32) -> bool {
        let Some(stage) = self.stages.get_mut(&stage_id) else {
            return true;
        };
        stage.set_group_size(group_index, size);
        false
    }

    // --- Getters for keeping state of new tournament & dependencies ---
    pub fn get_base(&self) -> &TournamentBase {
        &self.base
    }

    pub fn get_stage_by_number(&self, stage_number: u32) -> Option<&Stage> {
        let start = self.base.get_id();
        self.structure
            .edges_directed(start, Direction::Outgoing)
            .find_map(|(_source, target, edge)| {
                if let DependencyType::Stage = *edge
                    && let Some(stage) = self.stages.get(&target)
                    && stage.get_number() == stage_number
                {
                    Some(stage)
                } else {
                    None
                }
            })
    }

    pub fn get_stage_by_id(&self, stage_id: Uuid) -> Option<&Stage> {
        self.stages.get(&stage_id)
    }

    pub fn get_group_by_number(&self, stage_number: u32, group_number: u32) -> Option<&Group> {
        let Some(stage) = self.get_stage_by_number(stage_number) else {
            return None;
        };
        let stage_id = stage.get_id();

        self.structure
            .edges_directed(stage_id, Direction::Outgoing)
            .find_map(|(_source, target, edge)| {
                if let DependencyType::Group = *edge
                    && let Some(group) = self.groups.get(&target)
                    && group.get_number() == group_number
                {
                    Some(group)
                } else {
                    None
                }
            })
    }

    // --- Diff Collectors for Saving --- ToDo: do we still need this?

    pub fn collect_base_diff<'a>(&'a self, origin: &'a Tournament) -> Option<&'a TournamentBase> {
        (self.get_base() != origin.get_base()).then(|| self.get_base())
    }

    /// Returns modified or new stages that are currently linked in the graph structure.
    pub fn collect_stages_diff(
        &self,
        origin: &Tournament,
    ) -> <HashMap<Uuid, Stage> as Diffable<Stage>>::Diff {
        // We collect ALL valid reachable IDs in local. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'stages' map.
        let valid_ids = self.collect_ids_in_structure();

        self.stages.get_diff(&origin.stages, Some(&valid_ids))
    }

    pub fn collect_groups_diff(
        &self,
        origin: &Tournament,
    ) -> <HashMap<Uuid, Group> as Diffable<Group>>::Diff {
        // We collect ALL valid reachable IDs in local. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'groups' map.
        let valid_ids = self.collect_ids_in_structure();

        self.groups.get_diff(&origin.groups, Some(&valid_ids))
    }

    // --- Change Detection --- ToDo: do we still need this?

    /// Checks if there are any changes compared to the origin state.
    pub fn is_changed(&self, origin: &Tournament) -> bool {
        let start = self.base.get_id();

        // Check root
        if self.get_base() != origin.get_base() {
            return true;
        }

        // Traverse structure
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(object) = bfs.next(&self.structure) {
            for (_source, target, edge) in
                self.structure.edges_directed(object, Direction::Outgoing)
            {
                match edge {
                    DependencyType::Stage => {
                        let curr = self.stages.get(&target);
                        let orig = origin.stages.get(&target);
                        if curr != orig {
                            return true;
                        }
                    }
                    DependencyType::Group => {
                        let curr = self.groups.get(&target);
                        let orig = origin.groups.get(&target);
                        if curr != orig {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // --- Validation ---

    /// Validates the entire tournament structure.
    /// Returns Ok(()) if the entire structure represents a valid state that could be saved/started.
    /// Else returns all validation errors found.
    pub fn validate(&self) -> ValidationResult<()> {
        let mut errs = ValidationErrors::new();

        // 1. Root Tournament Check
        if let Err(err) = self.base.validate() {
            errs.append(err);
        }

        let start = self.base.get_id();

        // Traverse structure
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(object) = bfs.next(&self.structure) {
            for (_source, target, edge) in
                self.structure.edges_directed(object, Direction::Outgoing)
            {
                match edge {
                    DependencyType::Stage => {
                        // Stage needs Tournament context for validation (e.g. strict entrant limits)
                        if let Some(stage) = self.stages.get(&target)
                            && let Err(err) = stage.validate(&self.base)
                        {
                            errs.append(err);
                        };
                    }
                    DependencyType::Group => {
                        // ToDo: implement group validation
                        if let Some(_group) = self.groups.get(&target) {
                            continue;
                        }
                    }
                }
            }
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }

    /// Validates if the provided object numbers exist in the current tournament structure.
    /// We return Option<Vec<u32>>:
    /// - None: all object numbers are valid
    /// - Some(Vec<u32>): list of remaining valid object numbers found during validation
    /// Returning empty Vec means all provided numbers were invalid.
    pub fn validate_object_numbers(
        &self,
        stage_number: Option<u32>,
        group_number: Option<u32>,
        _round_number: Option<u32>,
        _match_number: Option<u32>,
    ) -> Option<Vec<u32>> {
        let start = self.base.get_id();
        let mut is_invalid = false;
        let mut valid_numbers = Vec::new();
        let mut queue: VecDeque<(Uuid, DependencyType)> = VecDeque::new();
        queue.push_back((start, DependencyType::Stage));

        // Traverse structure by existing dependencies and given params
        while let Some((current, dependency_type)) = queue.pop_front() {
            match dependency_type {
                DependencyType::Stage => {
                    let Some(sn) = stage_number else {
                        break;
                    };
                    // check if stage number is valid
                    if self.base.get_tournament_mode().get_num_of_stages() <= sn {
                        is_invalid = true;
                        break;
                    }
                    // valid stage number
                    valid_numbers.push(sn);
                    // add stage to queue, if it exists in state
                    if let Some(stage) = self.get_stage_by_number(sn) {
                        queue.push_back((stage.get_id(), DependencyType::Group));
                    }
                }
                DependencyType::Group => {
                    let Some(gn) = group_number else {
                        break;
                    };
                    // check if group number is valid
                    if let Some(stage) = self.stages.get(&current)
                        && stage.get_num_groups() <= gn
                    {
                        is_invalid = true;
                        break;
                    }
                    // valid group number
                    valid_numbers.push(gn);
                    // add group to queue, if it exists in state
                    // ToDo: implement group lookup by number
                }
            }
        }
        is_invalid.then_some(valid_numbers)
    }

    // --- Helpers ---

    /// Returns the root tournament ID.
    fn get_id(&self) -> Uuid {
        self.base.get_id()
    }

    /// Collects all valid IDs reachable from the root of structure.
    fn collect_ids_in_structure(&self) -> HashSet<Uuid> {
        let start = self.get_id();

        // Use BFS to traverse the entire dependency graph starting from root
        let bfs = Bfs::new(&self.structure, start);
        bfs.iter(&self.structure).collect()
    }

    /// Checks if the new tournament configuration requires removing stages.
    fn unlink_excess_stages(&mut self) {
        let root_id = self.base.get_id();
        let num_expected = self.base.get_tournament_mode().get_num_of_stages();

        let excess_ids =
            self.collect_excess_ids(root_id, DependencyType::Stage, &self.stages, num_expected);

        for stage_id in excess_ids {
            // We only remove the graph edge. The object remains in the Map until strictly cleared,
            // or we could remove it here. Removing edge hides it from the UI traversal.
            self.structure.remove_edge(root_id, stage_id);
        }
    }

    /// Checks if the new tournament configuration requires removing groups.
    fn unlink_excess_groups(&mut self, stage_id: Uuid) {
        if let Some(stage) = self.stages.get(&stage_id) {
            let num_expected = stage.get_num_groups();

            let excess_ids = self.collect_excess_ids(
                stage_id,
                DependencyType::Group,
                &self.groups,
                num_expected,
            );

            for group_id in excess_ids {
                // We only remove the graph edge. The object remains in the Map until strictly cleared,
                // or we could remove it here. Removing edge hides it from the UI traversal.
                self.structure.remove_edge(stage_id, group_id);
            }
        }
    }

    /// Collects IDs of dependent objects of a specific type that exceed a given limit.
    fn collect_excess_ids<O>(
        &self,
        parent_id: Uuid,
        dep_type: DependencyType,
        store: &HashMap<Uuid, O>,
        limit: u32,
    ) -> Vec<Uuid>
    where
        O: ObjectNumber + ObjectIdVersion,
    {
        self.structure
            .edges_directed(parent_id, Direction::Outgoing)
            .filter(|(_, _, e)| **e == dep_type)
            .filter_map(|(_, target, _)| store.get(&target).map(|obj| (target, obj)))
            .filter(|(_, obj)| obj.get_object_number() >= limit)
            .map(|(id, _)| id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_di_graph_map() {
        let mut graph: DiGraphMap<Uuid, DependencyType> = DiGraphMap::new();
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();
        graph.add_edge(node1, node2, DependencyType::Stage);
        graph.add_edge(node1, node3, DependencyType::Stage);

        println!("{}", serde_json::to_string_pretty(&graph).unwrap());
        let serialized = serde_json::to_string(&graph).unwrap();
        let deserialized: DiGraphMap<Uuid, DependencyType> =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(graph.all_edges().count(), deserialized.all_edges().count());
        for (source, target, edge) in graph.all_edges() {
            let found = deserialized
                .all_edges()
                .find(|(s, t, e)| *s == source && *t == target && *e == edge);
            assert!(found.is_some());
        }
    }

    #[test]
    fn test_serde_tournament() {
        let mut tournament = Tournament::new();
        let sport_id = Uuid::new_v4();
        tournament.new_base(sport_id);
        tournament.set_base_name("Test Tournament");
        tournament.set_base_num_entrants(16);
        tournament.set_base_mode(TournamentMode::PoolAndFinalStage);

        tournament.new_stage(0);
        tournament.new_stage(1);

        let stage0_id = tournament.get_stage_by_number(0).unwrap().get_id();
        let stage1_id = tournament.get_stage_by_number(1).unwrap().get_id();
        tournament.set_stage_number_of_groups(stage0_id, 4);
        tournament.set_stage_group_size(stage0_id, 0, 4);
        tournament.set_stage_group_size(stage0_id, 1, 4);
        tournament.set_stage_group_size(stage0_id, 2, 4);
        tournament.set_stage_group_size(stage0_id, 3, 4);

        tournament.set_stage_number_of_groups(stage1_id, 2);
        tournament.set_stage_group_size(stage1_id, 0, 8);
        tournament.set_stage_group_size(stage1_id, 1, 8);

        println!("{}", serde_json::to_string_pretty(&tournament).unwrap());
        let serialized = serde_json::to_string(&tournament).unwrap();
        let deserialized: Tournament = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tournament.base, deserialized.base);
        assert_eq!(tournament.stages.len(), deserialized.stages.len());
        for (id, stage) in &tournament.stages {
            let deserialized_stage = deserialized.stages.get(id).unwrap();
            assert_eq!(stage, deserialized_stage);
        }
    }
}
