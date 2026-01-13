use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IdVersion always provides a valid combination of id and version
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdVersion {
    New,
    NewWithId(Uuid),
    Existing(ExistingInner),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExistingInner {
    id: Uuid,
    version: u32,
}

impl ExistingInner {
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    pub fn get_version(&self) -> u32 {
        self.version
    }
}

impl IdVersion {
    pub fn new(id: Uuid, version: Option<u32>) -> IdVersion {
        if id.is_nil() {
            IdVersion::New
        } else if let Some(v) = version {
            IdVersion::Existing(ExistingInner { id, version: v })
        } else {
            IdVersion::NewWithId(id)
            
        }
    }
    pub fn get_initial_id(&self) -> Option<Uuid> {
        if let IdVersion::NewWithId(id) = self {
            Some(*id)
        } else {
            None
        }
    }
    pub fn get_id(&self) -> Option<Uuid> {
        if let IdVersion::Existing(inner) = self {
            Some(inner.id)
        } else {
            None
        }
    }
    pub fn get_version(&self) -> Option<u32> {
        if let IdVersion::Existing(inner) = self {
            Some(inner.version)
        } else {
            None
        }
    }
}

impl VersionId for IdVersion {
    fn get_id_version(&self) -> IdVersion {
        *self
    }
}

pub trait VersionId {
    fn get_id_version(&self) -> IdVersion;
}
