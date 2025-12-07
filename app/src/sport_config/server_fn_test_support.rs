use super::server_fn::save_sport_config_inner;
use crate::AppError;
use app_core::SportConfig;
use leptos::server_fn::{Protocol, ServerFn, client::Client, server::Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MockProtocol<SC, SS> {
    _phantom: std::marker::PhantomData<(SC, SS)>,
}

impl<SC, SS> Protocol<SaveSportConfig, SportConfig, SC, SS, AppError> for MockProtocol<SC, SS>
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
        F: Fn(SaveSportConfig) -> Fut + Send,
        Fut: Future<Output = Result<SportConfig, AppError>> + Send,
    {
        unimplemented!("MockProtocol cannot run server functions")
    }
    fn run_client(
        _path: &str,
        input: SaveSportConfig,
    ) -> impl Future<Output = Result<SportConfig, AppError>> + Send {
        input.run_body()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSportConfig {
    pub id: Uuid,
    pub version: u32,
    pub sport_id: Uuid,
    pub name: String,
    pub config: Value,
    pub intent: Option<String>,
}

impl ServerFn for SaveSportConfig {
    type Client = leptos::server_fn::client::browser::BrowserClient;
    type Server = leptos::server_fn::mock::BrowserMockServer;
    type Protocol = MockProtocol<Self::Client, Self::Server>;
    type Output = SportConfig;
    type Error = AppError;
    type InputStreamError = AppError;
    type OutputStreamError = AppError;

    const PATH: &'static str = "/mock_server";

    fn run_body(self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        save_sport_config_inner(
            self.id,
            self.version,
            self.sport_id,
            self.name,
            self.config,
            self.intent,
        )
    }
}
