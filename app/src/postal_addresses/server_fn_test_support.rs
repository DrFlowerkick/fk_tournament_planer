use super::server_fn::save_postal_address_inner;
use crate::AppError;
use app_core::PostalAddress;
use leptos::server_fn::{Protocol, ServerFn, client::Client, server::Server};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MockProtocol<SC, SS> {
    _phantom: std::marker::PhantomData<(SC, SS)>,
}

impl<SC, SS> Protocol<SavePostalAddress, PostalAddress, SC, SS, AppError> for MockProtocol<SC, SS>
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
        F: Fn(SavePostalAddress) -> Fut + Send,
        Fut: Future<Output = Result<PostalAddress, AppError>> + Send,
    {
        unimplemented!("MockProtocol cannot run server functions")
    }
    fn run_client(
        _path: &str,
        input: SavePostalAddress,
    ) -> impl Future<Output = Result<PostalAddress, AppError>> + Send {
        input.run_body()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavePostalAddress {
    pub id: Uuid,
    pub version: u32,
    pub name: String,
    pub street: String,
    pub postal_code: String,
    pub locality: String,
    pub region: Option<String>,
    pub country: String,
    pub intent: Option<String>,
}

impl ServerFn for SavePostalAddress {
    type Client = leptos::server_fn::client::browser::BrowserClient;
    type Server = leptos::server_fn::mock::BrowserMockServer;
    type Protocol = MockProtocol<Self::Client, Self::Server>;
    type Output = PostalAddress;
    type Error = AppError;
    type InputStreamError = AppError;
    type OutputStreamError = AppError;

    const PATH: &'static str = "/mock_server";

    fn run_body(self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        save_postal_address_inner(
            self.id,
            self.version,
            self.name,
            self.street,
            self.postal_code,
            self.locality,
            self.region,
            self.country,
            self.intent,
        )
    }
}
