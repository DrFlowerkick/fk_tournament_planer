//! preparing enums for usage as select options

use app_core::{TournamentMode, TournamentState, TournamentType};
use isocountry::CountryCode;
use std::{num::ParseIntError, str::FromStr};

pub trait SelectableOption: Sized + Clone + PartialEq + Send + Sync + 'static {
    /// Returns the unique string representation for the <option value="...">
    fn value(&self) -> String;

    /// Returns the display text for the UI
    fn label(&self) -> String;

    /// Returns all available options for the dropdown
    /// depending on the current value. This enables dynamic option lists,
    /// e.g. for TournamentState::ActiveStage(stage) we can return the current value
    /// of active stage in options (instead of a default value).
    fn options(&self) -> Vec<Self>;

    /// Static options for cases where we don't have a current value, e.g. if enum
    /// is wrapped into an Option and current value is None. This is a fallback for options() method.
    fn static_options() -> Vec<Self>;
}

impl SelectableOption for TournamentType {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        vec![TournamentType::Scheduled, TournamentType::Adhoc]
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
    }
}

impl SelectableOption for TournamentState {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        match self {
            TournamentState::ActiveStage(stage) => vec![
                TournamentState::Draft,
                TournamentState::Published,
                TournamentState::ActiveStage(*stage),
                TournamentState::Finished,
            ],
            _ => vec![
                TournamentState::Draft,
                TournamentState::Published,
                TournamentState::ActiveStage(0),
                TournamentState::Finished,
            ],
        }
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
    }
}

impl SelectableOption for TournamentMode {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        match self {
            TournamentMode::SwissSystem { num_rounds } => vec![
                TournamentMode::SingleStage,
                TournamentMode::PoolAndFinalStage,
                TournamentMode::TwoPoolStagesAndFinalStage,
                TournamentMode::SwissSystem {
                    num_rounds: *num_rounds,
                },
            ],
            _ => vec![
                TournamentMode::SingleStage,
                TournamentMode::PoolAndFinalStage,
                TournamentMode::TwoPoolStagesAndFinalStage,
                TournamentMode::SwissSystem { num_rounds: 0 },
            ],
        }
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
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

    fn options(&self) -> Vec<Self> {
        CountryCode::as_array().into()
    }

    fn static_options() -> Vec<Self> {
        Self::options(&CountryCode::DEU)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, displaydoc::Display)]
pub enum FilterLimit {
    #[default]
    /// 10
    Ten = 10,
    /// 20
    Twenty = 20,
    /// 50
    Fifty = 50,
    /// 100
    Hundred = 100,
}

impl FromStr for FilterLimit {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let u = s.parse::<u32>()?;
        match u {
            10 => Ok(FilterLimit::Ten),
            20 => Ok(FilterLimit::Twenty),
            50 => Ok(FilterLimit::Fifty),
            100 => Ok(FilterLimit::Hundred),
            _ => Err("invalid filter limit".parse::<u32>().unwrap_err()),
        }
    }
}

impl SelectableOption for FilterLimit {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        vec![
            FilterLimit::Ten,
            FilterLimit::Twenty,
            FilterLimit::Fifty,
            FilterLimit::Hundred,
        ]
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
    }
}
