// data types for postal addresses

use crate::Core;
use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Clone, Debug)]
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

/// view model of postal address
/// struct field description see PostalAddress
#[derive(Clone, Debug, Default)]
pub struct PostalAddressView {
    name: String,
    street_address: String,
    postal_code: String,
    address_locality: String,
    address_region: String,
    address_country: String,
}

impl From<PostalAddress> for PostalAddressView {
    fn from(value: PostalAddress) -> Self {
        PostalAddressView {
            name: value.name.clone().unwrap_or_default(),
            street_address: value.street_address.clone(),
            postal_code: value.postal_code.clone(),
            address_locality: value.address_locality.clone(),
            address_region: value.address_region.clone().unwrap_or_default(),
            address_country: value.address_country.clone(),
        }
    }
}

pub struct PostalAddressState {
    address: PostalAddress,
}

/// API of postal address
impl<S> Core<S> {
    pub async fn get_postal_address_state(
        &self,
        id: Uuid,
    ) -> Result<Option<Core<PostalAddressState>>> {
        if let Some(address) = self.data_base.get_postal_address(id).await? {
            // ToDo: client must register to registry
            return Ok(Some(self.switch_state(PostalAddressState { address })));
        }
        Ok(None)
    }
    pub fn new_postal_address_state(&self) -> Core<PostalAddressState> {
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
    pub async fn resync(&mut self) -> Result<&PostalAddress> {
        self.state.address = self
            .data_base
            .get_postal_address(self.state.address.id)
            .await?
            .context("Expected postal address")?;
        Ok(self.get())
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
    pub fn apply_view(&mut self, view: PostalAddressView) {
        self.state.address.name = (!view.name.is_empty()).then_some(view.name.to_string());
        self.state.address.street_address = view.street_address.to_string();
        self.state.address.postal_code = view.postal_code.to_string();
        self.state.address.address_locality = view.address_locality.to_string();
        self.state.address.address_region =
            (!view.address_region.is_empty()).then_some(view.address_region.to_string());
        self.state.address.address_country = view.address_country.to_string();
    }
    pub async fn save(&self) -> Result<PostalAddressView> {
        Ok(self
            .data_base
            .save_postal_address(&self.state.address)
            .await?
            .into())
    }
}
