// timing details of matches

use crate::Core;
use anyhow::{Context, Result};
use std::time::Duration;
use time::OffsetDateTime;
use uuid::Uuid;

/// Timing structure of a match. For set based sports with sets_to_win and
/// score_to_win (see crate::scoring::ScoringPolicy) the number of periods
/// may be set to the number of sets. The duration of a period has to be estimated
/// depending on score_to_win and experience with the particular sport.
#[derive(Debug, Clone)]
pub struct MatchTiming {
    /// number of periods (min 1)
    num_periods: u8,
    /// duration of a period
    duration_per_period: Duration,
    /// duration of the intervals between periods
    interval_between_periods: Duration,
}

#[derive(Debug, Clone)]
pub struct DayTiming {
    /// id of tournament day timing
    id: Uuid,
    /// id of schedule
    schedule_id: Uuid,
    /// number of day of tournament
    number: usize,
    /// date and start time of a tournament day
    pub date: OffsetDateTime,
    /// maximum duration of day
    max_duration: Duration,
    /// start of midday break
    midday_break: OffsetDateTime,
    /// midday break duration
    midday_break_duration: Duration,
}

pub struct DayTimingState {
    day_timing: DayTiming,
}

/// API of tournament day timings
impl<S> Core<S> {
    pub fn get_tournament_day_timing_state(
        &self,
        id: Uuid,
    ) -> Result<Option<Core<DayTimingState>>> {
        if let Some(day_timing) = self.load_tournament_day_timing(id)? {
            return Ok(Some(self.switch_state(DayTimingState { day_timing })));
        }
        Ok(None)
    }
    fn load_tournament_day_timing(&self, id: Uuid) -> Result<Option<DayTiming>> {
        todo!()
    }
}

impl Core<DayTimingState> {
    pub fn get(&self) -> &DayTiming {
        &self.state.day_timing
    }
    pub fn update(&mut self) -> Result<&DayTiming> {
        self.state.day_timing = self
            .load_tournament_day_timing(self.state.day_timing.id)?
            .context("Expected day timing")?;
        Ok(self.get())
    }
}
