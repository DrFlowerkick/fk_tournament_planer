//! server functions for tournament base entities

#[cfg(feature = "test-mock")]
pub mod test_support;
//#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppResult;
// IdVersion Import wird hier nicht mehr explizit benötigt, da der Client das Objekt fertig liefert
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::CoreState;
use app_core::TournamentBase;
use leptos::prelude::*;
#[cfg(not(feature = "test-mock"))]
use tracing::instrument;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use tracing::{error, info};
use uuid::Uuid;

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "tournament_base.load",
    skip_all,
    fields(id = %id)
)]
pub async fn load_tournament_base(id: Uuid) -> AppResult<Option<TournamentBase>> {
    load_tournament_base_inner(id).await
}

#[cfg(feature = "test-mock")]
pub async fn load_tournament_base(id: Uuid) -> AppResult<Option<TournamentBase>> {
    load_tournament_base_inner(id).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn load_tournament_base_inner(id: Uuid) -> AppResult<Option<TournamentBase>> {
    let mut core = expect_context::<CoreState>().as_tournament_base_state();
    let tb = core.load(id).await?.map(|tb| tb.to_owned());
    Ok(tb)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(name = "tournament_base.list", skip_all)]
pub async fn list_tournament_bases(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<TournamentBase>> {
    list_tournament_bases_inner(sport_id, name, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_tournament_bases(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<TournamentBase>> {
    list_tournament_bases_inner(sport_id, name, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_tournament_bases_inner(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<TournamentBase>> {
    let core = expect_context::<CoreState>().as_tournament_base_state();
    let configs = core
        .list_sport_tournaments(sport_id, Some(&name), limit)
        .await?;
    Ok(configs)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "tournament_base.save",
    skip_all,
    fields(
        id = ?tournament.get_id(),
        // We only log metadata, not complete payloads
        name_len = tournament.get_name().len(),
    )
)]
pub async fn save_tournament_base(tournament: TournamentBase) -> AppResult<TournamentBase> {
    save_tournament_base_inner(tournament).await
}

#[cfg(feature = "test-mock")]
pub use test_support::SaveTournamentBase;
#[cfg(feature = "test-mock")]
pub async fn save_tournament_base(tournament: TournamentBase) -> AppResult<TournamentBase> {
    save_tournament_base_inner(tournament).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn save_tournament_base_inner(tournament: TournamentBase) -> AppResult<TournamentBase> {
    let mut core = expect_context::<CoreState>().as_tournament_base_state();

    // We replace the state object in the core directly with the received object.
    // Prerequisite: The client has already set the correct IdVersion.
    *core.get_mut() = tournament;

    // Persist; log outcome with the saved id.
    match core.save().await {
        Ok(saved) => {
            // After saving, log the ID (especially important for new entries with generated ID)
            if let Some(id) = saved.get_id() {
                info!(saved_id = %id, "save_ok");
            } else {
                info!("save_ok_no_id_read"); // should not happen
            }
            Ok(saved.clone())
        }
        Err(e) => {
            error!(error = %e, "save_failed");
            Err(e.into())
        }
    }
}
