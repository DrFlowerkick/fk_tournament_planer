//! traits for utils

use crate::utils::id_version::IdVersion;

pub trait ObjectIdVersion {
    fn get_id_version(&self) -> IdVersion;
}

pub trait ObjectNumber {
    fn get_object_number(&self) -> u32;
}
