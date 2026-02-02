//! server fn to save tournament editor changes

#[cfg(feature = "ssr")]
use super::{stage::save_stage, tournament_base::save_tournament_base};
use crate::error::AppResult;
use app_core::{Stage, TournamentBase};
use leptos::{prelude::*, server_fn::codec::Json};
#[cfg(feature = "ssr")]
use tracing::info;
use tracing::instrument;
use uuid::Uuid;

#[server(input = Json, output = Json)]
#[instrument(name = "tournament_editor.save_diff", skip_all)]
pub async fn save_tournament_editor_diff(
    base_id: Uuid,
    base_diff: Option<TournamentBase>,
    stages_diff: Vec<Stage>,
) -> AppResult<Uuid> {
    if let Some(changed_base) = base_diff {
        info!("Saving tournament base changes");
        save_tournament_base(changed_base).await?;
    }
    for changed_stage in stages_diff {
        info!("Saving stage changes");
        save_stage(changed_stage).await?;
    }

    info!("All tournament editor changes saved");

    Ok(base_id)
}
