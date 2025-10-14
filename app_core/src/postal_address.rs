// data types for postal addresses

use crate::{Core, CrPushNotice, CrUpdateMeta, DbError, DbResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostalAddress {
    /// id of address
    pub id: Uuid,
    /// optimistic locking version
    pub version: i64,
    /// name of location
    pub name: Option<String>,
    /// street address
    pub street_address: String,
    /// postal code
    pub postal_code: String,
    /// city
    pub address_locality: String,
    /// optional region
    pub address_region: Option<String>,
    /// country: ISO name or code
    pub address_country: String,
}

impl Default for PostalAddress {
    fn default() -> Self {
        PostalAddress {
            id: Uuid::nil(),
            version: -1,
            name: None,
            street_address: "".into(),
            postal_code: "".into(),
            address_locality: "".into(),
            address_region: None,
            address_country: "".into(),
        }
    }
}

pub struct PostalAddressState {
    address: PostalAddress,
}

/// API of postal address
// switch state to portal address state to provide API for PostalAddress
impl<S> Core<S> {
    pub fn as_postal_address_state(&self) -> Core<PostalAddressState> {
        self.switch_state(PostalAddressState {
            address: PostalAddress::default(),
        })
    }
}

// ToDo: maybe add validations to change actions
impl Core<PostalAddressState> {
    pub fn get(&self) -> &PostalAddress {
        &self.state.address
    }
    pub fn set_id(&mut self, id: Uuid) {
        self.state.address.id = id;
    }
    pub fn set_version(&mut self, version: i64) {
        self.state.address.version = version;
    }
    pub fn change_name(&mut self, name: String) {
        self.state.address.name = (!name.is_empty()).then_some(name);
    }
    pub fn change_street_address(&mut self, street_address: String) {
        self.state.address.street_address = street_address;
    }
    pub fn change_postal_code(&mut self, postal_code: String) {
        self.state.address.postal_code = postal_code;
    }
    pub fn change_address_locality(&mut self, address_locality: String) {
        self.state.address.address_locality = address_locality;
    }
    pub fn change_address_region(&mut self, address_region: String) {
        self.state.address.address_region = (!address_region.is_empty()).then_some(address_region);
    }
    pub fn change_address_country(&mut self, address_country: String) {
        self.state.address.address_country = address_country;
    }
    pub async fn load(&mut self, id: Uuid) -> DbResult<Option<&PostalAddress>> {
        if let Some(address) = self.database.get_postal_address(id).await? {
            self.state.address = address;
            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }
    pub async fn resync(&mut self) -> DbResult<&PostalAddress> {
        self.state.address = self
            .database
            .get_postal_address(self.state.address.id)
            .await?
            .ok_or(DbError::NotFound)?;
        Ok(self.get())
    }
    pub async fn save(&mut self) -> DbResult<&PostalAddress> {
        self.state.address = self
            .database
            .save_postal_address(&self.state.address)
            .await?;
        // publish change of address to client registry
        let notice = CrPushNotice::AddressUpdated {
            id: self.state.address.id,
            meta: CrUpdateMeta {
                version: self.state.address.version,
            },
        };
        self.client_registry.publish(notice).await?;
        Ok(self.get())
    }
    pub async fn list_addresses(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<PostalAddress>> {
        self.database
            .list_postal_addresses(name_filter, limit)
            .await
    }
}

