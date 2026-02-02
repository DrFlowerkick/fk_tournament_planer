//! preparing enums for usage as select options

use app_core::{TournamentMode, TournamentState, TournamentType};
use isocountry::CountryCode;

pub trait SelectableOption: Sized + Clone + PartialEq + Send + Sync + 'static {
    /// Returns the unique string representation for the <option value="...">
    fn value(&self) -> String;

    /// Returns the display text for the UI
    fn label(&self) -> String;

    /// Returns all available options for the dropdown.
    /// For variants with data fields, return a default instance.
    fn options() -> Vec<Self>;
}

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
        format!("{}", self)
    }

    fn label(&self) -> String {
        format!("{}", self)
    }

    fn options() -> Vec<Self> {
        vec![
            TournamentState::Draft,
            TournamentState::Published,
            TournamentState::ActiveStage(0),
            TournamentState::Finished,
        ]
    }
}

impl SelectableOption for TournamentMode {
    fn value(&self) -> String {
        format!("{}", self)
    }

    fn label(&self) -> String {
        format!("{}", self)
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

/// SelectableOption implementation for CountryCode from isocountry crate
// Reason: we want to use CountryCode as select options in various places
impl SelectableOption for CountryCode {
    fn value(&self) -> String {
        self.alpha2().to_string()
    }

    fn label(&self) -> String {
        format!("{} ({})", self.name(), self.alpha2())
    }

    fn options() -> Vec<Self> {
        CountryCode::iter().cloned().collect()
    }
}
