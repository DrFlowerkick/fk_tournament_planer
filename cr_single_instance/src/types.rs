// type definitions shared by client and server

use app_core::CrTopic;
use displaydoc::Display;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Display)]
#[serde(rename_all = "kebab-case")]
pub enum CrKind {
    /// address
    Address,
}

impl From<&CrTopic> for CrKind {
    fn from(value: &CrTopic) -> Self {
        match value {
            CrTopic::Address(_) => CrKind::Address,
        }
    }
}

/// the url mus be identical to typed path URL of server::sse_service::CrTopicPath
pub const CR_TOPIC_URL_TEMPLATE: &str = "/api/cr/subscribe/{kind}/{id}";

pub trait SseUrl {
    fn sse_url(&self) -> String;
}

impl SseUrl for CrTopic {
    fn sse_url(&self) -> String {
        let id = match self {
            CrTopic::Address(id) => *id,
        };
        CR_TOPIC_URL_TEMPLATE
            .replace("{kind}", CrKind::from(self).to_string().as_str())
            .replace("{id}", id.to_string().as_ref())
    }
}
