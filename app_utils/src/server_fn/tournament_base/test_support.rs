use super::save_tournament_base_inner;
use crate::error::AppError;
use app_core::{TournamentBase, TournamentMode, TournamentState, TournamentType};
use leptos::server_fn::{Protocol, ServerFn, client::Client, server::Server};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MockProtocol<SC, SS> {
    _phantom: std::marker::PhantomData<(SC, SS)>,
}

impl<SC, SS> Protocol<SaveTournamentBase, TournamentBase, SC, SS, AppError> for MockProtocol<SC, SS>
where
    SS: Server<AppError>,
    SC: Client<AppError>,
{
    const METHOD: http::Method = http::Method::POST;

    async fn run_server<F, Fut>(
        _request: <SS as Server<AppError>>::Request,
        _server_fn: F,
    ) -> Result<<SS as Server<AppError>>::Response, AppError>
    where
        F: Fn(SaveTournamentBase) -> Fut + Send,
        Fut: Future<Output = Result<TournamentBase, AppError>> + Send,
    {
        unimplemented!("MockProtocol cannot run server functions")
    }
    fn run_client(
        _path: &str,
        input: SaveTournamentBase,
    ) -> impl Future<Output = Result<TournamentBase, AppError>> + Send {
        input.run_body()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveTournamentBase {
    pub id: Uuid,
    pub version: u32,
    pub name: String,
    pub sport_id: Uuid,
    pub num_entrants: u32,
    pub t_type: TournamentType,
    pub mode: TournamentMode,
    pub state: TournamentState,
    pub intent: Option<String>,
}

impl ServerFn for SaveTournamentBase {
    type Client = leptos::server_fn::client::browser::BrowserClient;
    type Server = leptos::server_fn::mock::BrowserMockServer;
    type Protocol = MockProtocol<Self::Client, Self::Server>;
    type Output = TournamentBase;
    type Error = AppError;
    type InputStreamError = AppError;
    type OutputStreamError = AppError;

    const PATH: &'static str = "/mock_server";

    fn run_body(self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        save_tournament_base_inner(
            self.id,
            self.version,
            self.name,
            self.sport_id,
            self.num_entrants,
            self.t_type,
            self.mode,
            self.state,
            self.intent,
        )
    }
}
