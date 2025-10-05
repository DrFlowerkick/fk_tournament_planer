// mock implementation of trait ClientRegistryPort

use std::sync::Arc;

use app_core::ClientRegistryPort;

pub struct ClientRegistryMock {}

impl ClientRegistryPort for ClientRegistryMock {}

impl ClientRegistryMock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}
