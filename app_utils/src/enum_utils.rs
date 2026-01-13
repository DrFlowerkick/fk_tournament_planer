//! preparing enums for usage as select options

use crate::components::inputs::SelectableOption;
use app_core::{TournamentMode, TournamentState, TournamentType};

impl SelectableOption for TournamentType {
    fn value(&self) -> String {
        format!("{:?}", self)
    }

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn options() -> Vec<Self> {
        vec![TournamentType::Scheduled, TournamentType::Adhoc]
    }
}

impl SelectableOption for TournamentState {
    fn value(&self) -> String {
        format!("{:?}", self)
    }

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn options() -> Vec<Self> {
        vec![
            TournamentState::Scheduling,
            TournamentState::Published,
            TournamentState::ActiveStage(0),
            TournamentState::Finished,
        ]
    }
}

impl SelectableOption for TournamentMode {
    fn value(&self) -> String {
        format!("{:?}", self)
    }

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn options() -> Vec<Self> {
        vec![
            TournamentMode::SingleStage,
            TournamentMode::PoolAndFinalStage,
            TournamentMode::TwoPoolStagesAndFinalStage,
            TournamentMode::SwissSystem { num_rounds: 0 },
        ]
    }
}
