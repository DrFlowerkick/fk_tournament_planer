use super::save_stage_inner;
use crate::error::AppError;
use app_core::Stage;
use leptos::server_fn::{Protocol, ServerFn, client::Client, server::Server};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct MockProtocol<SC, SS> {
    _phantom: std::marker::PhantomData<(SC, SS)>,
}

impl<SC, SS> Protocol<SaveStage, Stage, SC, SS, AppError> for MockProtocol<SC, SS>
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
        F: Fn(SaveStage) -> Fut + Send,
        Fut: Future<Output = Result<Stage, AppError>> + Send,
    {
        unimplemented!("MockProtocol cannot run server functions")
    }
    fn run_client(
        _path: &str,
        input: SaveStage,
    ) -> impl Future<Output = Result<Stage, AppError>> + Send {
        input.run_body()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveStage {
    pub stage: Stage,
}

impl ServerFn for SaveStage {
    type Client = leptos::server_fn::client::browser::BrowserClient;
    type Server = leptos::server_fn::mock::BrowserMockServer;
    type Protocol = MockProtocol<Self::Client, Self::Server>;
    type Output = Stage;
    type Error = AppError;
    type InputStreamError = AppError;
    type OutputStreamError = AppError;

    const PATH: &'static str = "/mock_server";

    fn run_body(self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        save_stage_inner(self.stage)
    }
}
