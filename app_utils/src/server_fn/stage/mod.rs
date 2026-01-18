//! server functions for stage entities

#[cfg(feature = "test-mock")]
pub mod test_support;
//#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppResult;
// IdVersion Import wird hier nicht mehr explizit benötigt, da der Client das Objekt fertig liefert
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::CoreState;
use app_core::Stage;
use leptos::prelude::*;
#[cfg(not(feature = "test-mock"))]
use tracing::instrument;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use tracing::{error, info};
use uuid::Uuid;

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "stage.load_by_id",
    skip_all,
    fields(id = %id)
)]
pub async fn load_stage_by_id(tournament_id: Uuid, id: Uuid) -> AppResult<Option<Stage>> {
    load_stage_by_id_inner(tournament_id, id).await
}

#[cfg(feature = "test-mock")]
pub async fn load_stage_by_id(tournament_id: Uuid, id: Uuid) -> AppResult<Option<Stage>> {
    load_stage_by_id_inner(tournament_id, id).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn load_stage_by_id_inner(tournament_id: Uuid, id: Uuid) -> AppResult<Option<Stage>> {
    let mut core = expect_context::<CoreState>()
        .as_stage_state(tournament_id)
        .await?;
    let tb = core.load_by_id(id).await?.map(|tb| tb.to_owned());
    Ok(tb)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "stage.load_by_number",
    skip_all,
    fields(number = %number)
)]
pub async fn load_stage_by_number(tournament_id: Uuid, number: u32) -> AppResult<Option<Stage>> {
    load_stage_by_number_inner(tournament_id, number).await
}

#[cfg(feature = "test-mock")]
pub async fn load_stage_by_number(tournament_id: Uuid, number: u32) -> AppResult<Option<Stage>> {
    load_stage_by_number_inner(tournament_id, number).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn load_stage_by_number_inner(
    tournament_id: Uuid,
    number: u32,
) -> AppResult<Option<Stage>> {
    let mut core = expect_context::<CoreState>()
        .as_stage_state(tournament_id)
        .await?;
    let tb = core.load_by_number(number).await?.map(|tb| tb.to_owned());
    Ok(tb)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(name = "stage.list_all_of_tournament", skip_all)]
pub async fn list_stages_of_tournament(tournament_id: Uuid) -> AppResult<Vec<Stage>> {
    list_stages_of_tournament_inner(tournament_id).await
}

#[cfg(feature = "test-mock")]
pub async fn list_stages_of_tournament(tournament_id: Uuid) -> AppResult<Vec<Stage>> {
    list_stages_of_tournament_inner(tournament_id).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_stages_of_tournament_inner(tournament_id: Uuid) -> AppResult<Vec<Stage>> {
    let core = expect_context::<CoreState>()
        .as_stage_state(tournament_id)
        .await?;
    let stages = core.list_stages_of_tournament().await?;
    Ok(stages)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "stage.save",
    skip_all,
    fields(
        id = ?stage.get_id(),
        number = stage.get_number(),
    )
)]
pub async fn save_stage(stage: Stage) -> AppResult<Stage> {
    save_stage_inner(stage).await
}

#[cfg(feature = "test-mock")]
pub use test_support::SaveStage;
#[cfg(feature = "test-mock")]
pub async fn save_stage(stage: Stage) -> AppResult<Stage> {
    save_stage_inner(stage).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn save_stage_inner(stage: Stage) -> AppResult<Stage> {
    let mut core = expect_context::<CoreState>()
        .as_stage_state(stage.get_tournament_id())
        .await?;

    // We replace the state object in the core directly with the received object.
    // Prerequisite: The client has already set the correct IdVersion.
    *core.get_mut() = stage;
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
