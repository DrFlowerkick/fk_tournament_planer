use crate::utils::traits::ObjectIdVersion;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IdVersion always provides a valid combination of id and version
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdVersion {
    NewWithId(Uuid),
    Existing(ExistingInner),
}

impl Default for IdVersion {
    fn default() -> Self {
        IdVersion::NewWithId(Uuid::new_v4())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExistingInner {
    id: Uuid,
    version: u32,
}

impl ExistingInner {
    pub fn get_id(&self) -> Uuid {
        self.id
    }
    pub fn get_version(&self) -> u32 {
        self.version
    }
}

impl IdVersion {
    pub fn new(id: Uuid, version: Option<u32>) -> IdVersion {
        if id.is_nil() {
            IdVersion::NewWithId(Uuid::new_v4())
        } else if let Some(v) = version {
            IdVersion::Existing(ExistingInner { id, version: v })
        } else {
            IdVersion::NewWithId(id)
        }
    }
    pub fn get_id(&self) -> Uuid {
        match self {
            IdVersion::Existing(inner) => inner.id,
            IdVersion::NewWithId(id) => *id,
        }
    }
    pub fn get_version(&self) -> Option<u32> {
        if let IdVersion::Existing(inner) = self {
            Some(inner.version)
        } else {
            None
        }
    }
    pub fn is_new(&self) -> bool {
        matches!(self, IdVersion::NewWithId(_))
    }
}

impl ObjectIdVersion for IdVersion {
    fn get_id_version(&self) -> IdVersion {
        *self
    }
}
