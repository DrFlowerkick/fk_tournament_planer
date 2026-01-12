//! server functions for tournament base entities

#[cfg(feature = "test-mock")]
pub mod test_support;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppError;
use crate::error::AppResult;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{CoreState, utils::id_version::IdVersion};
use app_core::{TournamentBase, TournamentMode, TournamentState, TournamentType};
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
) -> AppResult<Vec<app_core::TournamentBase>> {
    list_tournament_bases_inner(sport_id, name, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_tournament_bases(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<app_core::TournamentBase>> {
    list_tournament_bases_inner(sport_id, name, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_tournament_bases_inner(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<app_core::TournamentBase>> {
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
        id = %id,
        version = version,
        // capture intent without logging full payloads
        intent = intent.as_deref().unwrap_or(""),
        // tiny hints only; avoid PII/body dumps
        name_len = name.len(),
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn save_tournament_base(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form
    version: u32,
    name: String,
    sport_id: Uuid,
    num_entrants: u32,
    t_type: TournamentType,
    mode: TournamentMode,
    state: TournamentState,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<TournamentBase> {
    save_tournament_base_inner(
        id,
        version,
        name,
        sport_id,
        num_entrants,
        t_type,
        mode,
        state,
        intent,
    )
    .await
}

#[cfg(feature = "test-mock")]
pub use test_support::SaveTournamentBase;
#[cfg(feature = "test-mock")]
pub async fn save_tournament_base(
    id: Uuid,
    version: u32,
    name: String,
    sport_id: Uuid,
    num_entrants: u32,
    t_type: TournamentType,
    mode: TournamentMode,
    state: TournamentState,
    intent: Option<String>,
) -> AppResult<TournamentBase> {
    save_tournament_base_inner(
        id,
        version,
        name,
        sport_id,
        num_entrants,
        t_type,
        mode,
        state,
        intent,
    )
    .await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
#[allow(clippy::too_many_arguments)]
pub async fn save_tournament_base_inner(
    id: Uuid,
    version: u32,
    name: String,
    sport_id: Uuid,
    num_entrants: u32,
    t_type: TournamentType,
    mode: TournamentMode,
    state: TournamentState,
    intent: Option<String>,
) -> AppResult<TournamentBase> {
    let mut core = expect_context::<CoreState>().as_tournament_base_state();

    // get mut handle to wrapped TournamentBase
    let mut_sc_core = core.get_mut();

    // Interpret intent
    let is_update = matches!(intent.as_deref(), Some("update"));
    if is_update {
        // set id and version previously loaded
        if id.is_nil() {
            return Err(AppError::NilIdUpdate);
        }
        let id_version = IdVersion::new(id, version);
        mut_sc_core.set_id_version(id_version);
        info!("saving_update");
    } else {
        info!("saving_create");
    }

    // set sport config data from Form inputs
    mut_sc_core
        .set_name(name)
        .set_sport_id(sport_id)
        .set_num_entrants(num_entrants)
        .set_tournament_type(t_type)
        .set_tournament_mode(mode)
        .set_tournament_state(state);

    // Persist; log outcome with the saved id. if save() is ok, it returns valid id -> unwrap() is save
    match core.save().await {
        Ok(saved) => {
            info!(saved_id = %saved.get_id().unwrap(), "save_ok");
            Ok(saved.clone())
        }
        Err(e) => {
            // Primary goal failed -> error
            error!(error = %e, "save_failed");
            Err(e.into())
        }
    }
}
