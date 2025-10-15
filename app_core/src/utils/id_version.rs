//!
//!
use uuid::Uuid;

/// ID and Version builder to force valid uuid and version combination when
/// creating new objects with private id and version fields
pub struct NoId {}
pub struct NoVersion {}

pub struct IdVersionBuilder<ID, VE> {
    id: ID,
    version: VE,
}

impl IdVersionBuilder<NoId, NoVersion> {
    pub fn build(self) -> IdVersion {
        IdVersion {
            id: Uuid::nil(),
            version: -1,
        }
    }
    pub fn set_id(self, id: Uuid) -> Result<IdVersionBuilder<Uuid, NoVersion>, IdVersion> {
        if id.is_nil() {
            Err(self.build())
        } else {
            Ok(IdVersionBuilder {
                id,
                version: self.version,
            })
        }
    }
}

impl IdVersionBuilder<Uuid, NoVersion> {
    pub fn set_version(self, version: u64) -> IdVersionBuilder<Uuid, i64> {
        assert!(version <= i64::MAX as u64);
        IdVersionBuilder {
            id: self.id,
            version: version as i64,
        }
    }
}

impl IdVersionBuilder<Uuid, i64> {
    pub fn build(self) -> IdVersion {
        IdVersion {
            id: self.id,
            version: self.version,
        }
    }
}

/// IdVersion always provides a valid combination of id and version
pub struct IdVersion {
    id: Uuid,
    version: i64,
}

impl IdVersion {
    pub fn builder() -> IdVersionBuilder<NoId, NoVersion> {
        IdVersionBuilder {
            id: NoId {},
            version: NoVersion {},
        }
    }
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    pub fn get_version(&self) -> &i64 {
        &self.version
    }
}
