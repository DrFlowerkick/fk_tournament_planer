/// Tournament functionality
///
/// This tournament app aims at sports, where teams or individual athletes compete
/// with each other in direct matches, which can either be won or lost or may
/// end in a draw (depends upon the respective sport). In context of this app
/// these competitors are called entrants of the tournament.
///
/// A tournament consists of one or more stages.
///
/// In each stage entrants are grouped in one or more groups. In first stage,
/// group mapping of entrants may be done by rank (e.g. world rank), equal
/// distributed rank (a.k.a. "counting trough": if you have 20 entrants and
/// 5 groups, you count trough from top rank to lowest rank from 1 to 5,
/// in which the number represents the mapped group), or random. When moving
/// to next stage, current stage rank decides group mapping (see below).
/// Snake muster anschauen. Eher der Standard international.
/// ToDo: die number of courts können über das Turnier ggf. variieren.
/// Knock Out mit Wild Card? Recherchieren...
/// Ein Turnierort sollte optional sein, für quick and dirty tournaments
/// Dafür sollten dann auh einfach dummy entrants möglich sein, die einfach per Nummer durchgezählt sind.
///
/// I each group all entrants have matches against each other after a
/// certain mode. In final stage this is usually KO Play Out (see below), while
/// earlier stages use round robin. The mode depends upon group size. KO Play Out
/// requires a group size of 2^n with n= 1, 2, 3, ... entrants. These matches are
/// organized in rounds. With an even number of entrants in group, each entrant
/// has one match per round (if not dropped out of tournament, see KO below).
/// An odd number of entrants implies a pause for one entrant in each round.
/// ToDo: Zuordnung zu de stations: die top gesetzten Teams werden festen Stationen zugeteilt,
/// die anderen müssen wandern. StationPolicy?
///
/// After all rounds of matches in all groups of current stage are done, a stage
/// ranking is generated for all entrants. This ranking depends upon ranking of
/// previous stage, if any, and results of current stage. Ranking of previous stage
/// is represented by group number of current stage. Depending upon stage ranking
/// the entrants are mapped to groups of next stage: if you have 20 entrants
/// and 5 groups in next stage, the top 4 are mapped to group 1, next top 4
/// to group 2, etc.. Final ranking of tournament is equal to ranking of
/// final stage.
///
/// Tournaments with [Swiss system](https://en.wikipedia.org/wiki/Swiss-system_tournament)
/// work different. Swiss system is similar to round robin, but instead of a full
/// round robin each entrant has a certain number of matches less than (number of entrants - 1).
/// After each round of matches ranking is resolved. Match partners of next round
/// depend upon current ranking. The core idea is, that current best entrant has his next
/// match against the next best entrant, which it has not faced yet. Same is done for
/// the remaining entrants until all entrants have a match partner in next round.
/// In case of odd number of entrants, one entrant gains a "free win" and pauses
/// for one round. Each entrant may have only one free win. After certain number of
/// rounds the ranking of entrants is stable enough to represent the final ranking.
/// The recommended number of rounds to play is 'log_2(number of entrants) + 2' or more.
/// The maximum number of rounds is equal to round robin.
/// Hier ggf. mit Buffern arbeiten. Nochmal recherchieren.
/// Double elimination wird durchaus verwendet (z.B. Free Style)
/// Ring System. Man stellt die Mannschaften in einem Ring auf und spielt gegen die Nachbarn
/// -> Recherchieren
///
/// Harte Time caps im Timing ergänzen
/// Noch nicht gestartete stages sollten auch bei gestarteten Turnier nur bearbeitbar im schedule sein.
/// Tie Breaker sollen durch den turnierdirektor konfigurierbar sein.
///
/// The Swiss system can be integrated in the generic tournament structure by using a stage
/// for each round of Swiss matches. Therefore a Swiss stage contains only one
/// group (all entrants) and one round of matches.
///
/// Depending upon tournament style, some entrants may drop out of tournament
/// after each stage or during KO mode. KO mode is normally used in final stage,
/// if at all. KO vs KO Play Out: in KO the loser drops out of tournament,
/// while in KO Play Out the losers match against each other to play out
/// lower ranking.
///
/// Ranking is resolved usually by comparing wins, losses, and draws, if applicable.
/// Normally wins and draws give some amount of victory points (e.g. 1 for wins and
/// 0.5 for draws). Most victory points result in best rank. Ranks start from 1 (best
/// rank) to n (last rank) with n being the number of entrants. In all stages after the first,
/// ranks are first resolved in each group and than ordered by group number:
/// if you have 20 entrants and 5 groups, rank 1 to 4 of first group have rank 1 to 4
/// of all entrants, rank 1 to 4 of second group have rank 5 to 8 of all entrants, etc..
///
/// There are several ways to break ties, e.g.
/// - delta points
/// - total points
/// - direct comparison (did entrants face each other? If yes, who won?)
/// - initial tournament ranking (e.g. inferred from world ranking system)
/// - coin flip
///
/// Swiss mode uses [buchholz score](https://en.wikipedia.org/wiki/Buchholz_system)
/// or something similar as primary way of breaking ties. These (and more) options can be
/// combined.
///
///
/// data structures
///
/// One option may be to put all parameters in one big struct. Since this will get more
/// and more confusing with growing size of struct, I suggest separate structs for the
/// components of the tournament, which data will be persisted via  database.
///
/// Tournament is structured into 4 main parts (which my change later):
/// 1. tournament base: sporting type, mode, number of entrants, and status
/// 2. tournament structure: stages, groups, matches
/// 3. tournament schedule: dates and times of tournament days, stages, matches
/// 4. tournament organization: name, location, stations, officials
/// For a simple adhoc tournament only parts 1 and 2 are required.
pub mod base;
pub mod editor;
pub mod stage;

pub use base::*;
pub use editor::*;
pub use stage::*;

use crate::{
    CoreError, CoreResult, Group,
    utils::{
        id_version::IdVersion,
        traits::{Diffable, ObjectIdVersion, ObjectNumber},
    },
};
use petgraph::{Direction, graphmap::DiGraphMap, visit::Bfs};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DependencyType {
    Stage,
    Group,
}

/// Tournament holds all data of a tournament. It data structure relies on the
/// following concepts:
/// - Data of a tournament is composed of multiple objects: tournament base,
///   stages, groups, matches, etc. Each object has its own struct representation. Setters
///   and getters in Tournament allow adding and retrieving these objects individually.
/// - Unique IDs (UUIDs) for all objects: every object added to the tournament
///   structure must have an unique ID. This allows easy referencing between objects
///   and prevents duplication.
/// - Heritage of objects is bottom up: each object knows its own ID and the ID of its
///   parent object (e.g. group knows its stage ID). This allows easy traversal
///   from child to parent, which is useful for validation and consistency checks.
/// - Each dependent object (stage, group, etc.) has besides its ID a number
///   (e.g. stage number, group number) that represents its position within its
///   parent object. Since heritage is bottom up, top down traversal (e.g. from tournament
///   to stages) is done by object number and given parent ID.
/// - Database persistence is done per object type: each object type may be
///   persisted in its own database table. This allows easy querying and updating
///   of individual objects. It also prevents big data payloads when only a subset of
///   objects need to be loaded or updated.
/// - Directed Graph for dependencies: handling objects with bottom up heritage is kinda
///   tricky. Therefore we use internally a directed graph (using petgraph crate) to
///   represent the the structure of the tournament. The graph is a directed acyclic
///   graph, where nodes are objects (tournament, stages, groups, etc.) and edges
///   represent dependencies (e.g. tournament -> stage, stage -> group). This allows
///   flexible representation of complex tournament structures and easy traversal.
///   The graph represents the "truth" of the tournament structure.
/// - The ID of the tournament base object serves as root node of the graph. All other
///   objects are reachable from this root node via directed edges. Changing the root
///   ID (e.g. when loading a different tournament) will reset the entire structure.
/// - Some changes to the tournament may make dependent objects invalid (e.g. changing
///   tournament mode to one with less stages). In such cases the structure graph
///   is updated to remove edges to now invalid objects. The objects node remains in
///   the graph unconnected to their former parent. The objects themselves remain in
///   the respective HashMaps, keeping them available for potential future use.
/// - HashMaps for object storage: actual objects are stored in HashMaps, keyed by
///   their UUIDs. This allows efficient retrieval and modification of objects.
///   The HashMaps are secondary storage, while the graph represents the structure.
///
/// To create a new tournament, simply create a new Tournament instance and use
/// the setters to add and update objects. The graph structure will be updated automatically.
/// One side effect of this design is, that objects may exist in the HashMaps,
/// but their nodes in structure are not reachable when traversing from the root node. This
/// may happen, if either adding objects without their parents being part of the structure,
/// or when invalidating objects due to changes in the tournament (see above). These
/// unreachable objects are effectively "orphaned" and will not be considered
/// in validation, saving, or other operations that traverse the structure.
/// One may think "Just remove these orphaned objects from the HashMaps to free up memory".
/// While this is possible, it may lead to unexpected behavior. For example, if you
/// add for some reason first a group and later its parent stage, the group
/// would be removed when adding the stage, since it was orphaned before.
/// This means that you must be careful with removing "orphaned" objects from the structure
/// and HashMaps. Only do this, if you are sure, that there are no "parallel" references
/// to these objects from other parts of the application, which may add an object to tournament,
/// which would connect a seemingly "orphaned" object back into the structure.
/// E.g., if you use leptos reactive system as ui framework and provide Tournament as
/// an app wide state, multiply components may interact with the tournament state.
/// Since you cannot control the order of operations in different components, you
/// must be careful with removing objects from structure and HashMaps, as other components
/// may still reference these objects and add them back into the structure.
/// Conclusion: it is better to leave "orphaned" objects in structure and HashMaps, as long
/// as you are not creating tournaments with hundreds of stages or groups, which would
/// consume too much memory.
#[derive(Clone, Deserialize, Serialize)]
pub struct Tournament {
    /// base of tournament
    pub base: Option<TournamentBase>,
    /// map of tournament dependencies
    pub structure: DiGraphMap<Uuid, DependencyType>,
    /// stages associated with the tournament
    pub stages: HashMap<Uuid, Stage>,
    /// groups associated with stages (not yet used)
    pub groups: HashMap<Uuid, Group>,
}

impl Tournament {
    pub fn new() -> Self {
        Tournament {
            base: None,
            structure: DiGraphMap::new(),
            stages: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    // --- Generators for empty objects ---

    /// Creates a new tournament base with a new ID and adds it to the tournament.
    pub fn new_base(&mut self) -> &TournamentBase {
        let base = TournamentBase::new(IdVersion::new(Uuid::new_v4(), None));
        // unwrap is safe here, as we just created a valid base with ID
        self.set_base(base).unwrap()
    }

    /// Creates a new stage with a new ID and adds it to the tournament.
    pub fn new_stage(&mut self, stage_number: u32) -> CoreResult<&Stage> {
        if self.get_stage_by_number(stage_number).is_some() {
            // return existing stage
            self.get_stage_by_number(stage_number)
                .ok_or(CoreError::MissingId("Stage".into()))
        } else {
            let tournament = self.expect_base()?;
            let tournament_id = self.expect_tournament_id()?;
            let stage_id = Uuid::new_v4();
            let mut stage = Stage::new(IdVersion::new(stage_id, None));
            // only allow valid stage number
            stage
                .set_number(stage_number)
                .set_tournament_id(tournament_id);
            // validate stage before adding to validate stage_number
            stage.validate(tournament)?;
            self.set_stage(stage)
        }
    }

    // ---- Setters ----

    /// Setter for the tournament base. Expects a valid base with ID.
    pub fn set_base(&mut self, base: TournamentBase) -> CoreResult<&TournamentBase> {
        let new_id = base.get_id_version().get_id();

        // only allow valid base
        base.validate()?;

        // Ensure Graph Node exists
        self.structure.add_node(new_id);

        // Set base
        self.base = Some(base);

        // Validation: Check if changes invalidate child objects (e.g. Mode change -> fewer stages)
        self.cleanup_excess_stages(new_id);
        self.expect_base()
    }

    pub fn set_base_name(&mut self, name: impl Into<String>) -> CoreResult<()> {
        self.expect_base_mut()?.set_name(name);
        Ok(())
    }

    pub fn set_base_num_entrants(&mut self, num_entrants: u32) -> CoreResult<()> {
        self.expect_base_mut()?.set_num_entrants(num_entrants);
        Ok(())
    }

    pub fn set_base_mode(&mut self, mode: TournamentMode) -> CoreResult<()> {
        self.expect_base_mut()?.set_tournament_mode(mode);
        Ok(())
    }

    pub fn clear_base(&mut self) {
        self.base = None;
    }

    /// Adds a stage to the state and links it to the tournament.
    pub fn set_stage(&mut self, stage: Stage) -> CoreResult<&Stage> {
        let stage_id = stage.get_id_version().get_id();
        // only allow valid stage
        let tournament = self.expect_base()?;
        stage.validate(tournament)?;

        // check for existing stage with same number but different ID
        let tournament_id = self.expect_tournament_id()?;
        if let Some(stage) = self.get_stage_by_number(stage.get_number())
            && stage.get_id() != stage_id
        {
            // remove edge to existing stage with same number but different ID from structure
            // If the existing stage is already persisted in database, this will result in an
            // error during saving, since unique constraint on (tournament_id, stage_number) will be violated.
            self.structure.remove_edge(tournament_id, stage.get_id());
        }

        // Link to tournament root, which although adds the node if missing
        self.structure
            .add_edge(tournament_id, stage_id, DependencyType::Stage);

        // Add to stages map
        self.stages.insert(stage_id, stage);

        // Validation: Check if changes invalidate child objects (e.g. fewer groups)
        self.cleanup_excess_groups(stage_id);
        self.stages
            .get(&stage_id)
            .ok_or(CoreError::MissingId("Stage".into()))
    }

    pub fn set_stage_number_of_groups(
        &mut self,
        stage_id: Uuid,
        num_groups: u32,
    ) -> CoreResult<()> {
        let Some(stage) = self.stages.get_mut(&stage_id) else {
            return Err(CoreError::MissingId("Stage".into()));
        };
        stage.set_num_groups(num_groups);

        Ok(())
    }

    // --- Getters for keeping state of new tournament & dependencies ---
    pub fn get_base(&self) -> Option<&TournamentBase> {
        self.base.as_ref()
    }

    pub fn get_stage_by_number(&self, stage_number: u32) -> Option<&Stage> {
        let Some(start) = self.get_root_id() else {
            return None;
        };
        for (_source, target, edge) in self.structure.edges_directed(start, Direction::Outgoing) {
            if let DependencyType::Stage = *edge
                && let Some(stage) = self.stages.get(&target)
                && stage.get_number() == stage_number
            {
                return Some(stage);
            }
        }
        None
    }

    // --- Diff Collectors for Saving ---

    pub fn collect_base_diff<'a>(
        &'a self,
        origin: &'a Tournament,
    ) -> <Option<&'a TournamentBase> as Diffable<&'a TournamentBase>>::Diff {
        // Option diffing ignores the filter
        self.get_base().get_diff(&origin.get_base(), None)
    }

    /// Returns modified or new stages that are currently linked in the graph structure.
    pub fn get_stages_diff(
        &self,
        origin: &Tournament,
    ) -> <HashMap<Uuid, Stage> as Diffable<Stage>>::Diff {
        // We collect ALL valid reachable IDs in local. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'stages' map.
        let valid_ids = self.get_valid_dependencies();

        self.stages.get_diff(&origin.stages, Some(&valid_ids))
    }

    pub fn get_groups_diff(
        &self,
        origin: &Tournament,
    ) -> <HashMap<Uuid, Group> as Diffable<Group>>::Diff {
        // We collect ALL valid reachable IDs in local. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'groups' map.
        let valid_ids = self.get_valid_dependencies();

        self.groups.get_diff(&origin.groups, Some(&valid_ids))
    }

    // --- Change Detection ---

    /// Checks if there are any changes compared to the origin state.
    pub fn is_changed(&self, origin: &Tournament) -> bool {
        let Some(start) = self.get_root_id() else {
            return false;
        };

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

    /// Collects all valid IDs reachable from the root of structure.
    pub fn get_valid_dependencies(&self) -> HashSet<Uuid> {
        let mut valid = HashSet::new();
        let Some(start) = self.get_root_id() else {
            return valid;
        };

        // Use BFS to traverse the entire dependency graph starting from root
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(node) = bfs.next(&self.structure) {
            // We include all reachable nodes.
            // Since UUIDs are unique, we don't need to filter by type here.
            // The specific HashMaps (stages, groups, etc.) will simply ignore
            // IDs that belong to other types when we try to look them up later.
            valid.insert(node);
        }

        valid
    }

    /// Validates the entire tournament structure.
    /// Returns Ok(()) if the entire structure represents a valid state that could be saved/started.
    /// Else returns all validation errors found.
    pub fn is_valid(&self) -> bool {
        // 1. Root Tournament Check
        let Some(tournament) = self.get_base() else {
            return false;
        };

        // Assuming TournamentBase has a validate() method returning Result
        if tournament.validate().is_err() {
            return false;
        }

        let Some(start) = self.get_root_id() else {
            return false;
        };

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
                            && stage.validate(tournament).is_err()
                        {
                            return false;
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

        true
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
        let mut valid_numbers = Vec::new();
        let Some(tournament) = self.get_base() else {
            return Some(valid_numbers);
        };
        let Some(start) = self.get_root_id() else {
            return Some(valid_numbers);
        };
        let mut is_invalid = false;
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
                    if tournament.get_tournament_mode().get_num_of_stages() <= sn {
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

    /// Returns the root tournament ID, returning None if base is not set.
    fn get_root_id(&self) -> Option<Uuid> {
        self.get_base().map(|t| t.get_id())
    }

    /// Expects the base tournament, returning an error if not set.
    fn expect_base(&self) -> CoreResult<&TournamentBase> {
        self.get_base()
            .ok_or(CoreError::MissingId("TournamentBase".into()))
    }

    /// Expects the mutable base tournament, returning an error if not set.
    fn expect_base_mut(&mut self) -> CoreResult<&mut TournamentBase> {
        self.base
            .as_mut()
            .ok_or(CoreError::MissingId("TournamentBase".into()))
    }

    /// Expects the root tournament ID, returning an error if base is not set.
    fn expect_tournament_id(&self) -> CoreResult<Uuid> {
        self.get_root_id()
            .ok_or(CoreError::MissingId("TournamentBase".into()))
    }

    /// Checks if the new tournament configuration requires removing stages.
    fn cleanup_excess_stages(&mut self, root_id: Uuid) {
        if let Some(tournament) = self.get_base() {
            let num_expected = tournament.get_tournament_mode().get_num_of_stages();

            let excess_ids =
                self.collect_excess_ids(root_id, DependencyType::Stage, &self.stages, num_expected);

            for stage_id in excess_ids {
                // We only remove the graph edge. The object remains in the Map until strictly cleared,
                // or we could remove it here. Removing edge hides it from the UI traversal.
                self.structure.remove_edge(root_id, stage_id);
            }
        }
    }

    /// Checks if the new tournament configuration requires removing stages.
    fn cleanup_excess_groups(&mut self, root_id: Uuid) {
        if let Some(stage) = self.stages.get(&root_id) {
            let num_expected = stage.get_num_groups();

            let excess_ids =
                self.collect_excess_ids(root_id, DependencyType::Group, &self.groups, num_expected);

            for group_id in excess_ids {
                // We only remove the graph edge. The object remains in the Map until strictly cleared,
                // or we could remove it here. Removing edge hides it from the UI traversal.
                self.structure.remove_edge(root_id, group_id);
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
