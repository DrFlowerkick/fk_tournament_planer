// data types for postal addresses

use crate::{
    Core, CoreResult, CrMsg, CrTopic,
    utils::{id_version::IdVersion, normalize::*, traits::ObjectIdVersion, validation::*},
};
// ToDo: should we us isocountry::CountryCode here for country field?
use isocountry::CountryCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PostalAddress {
    /// id and optimistic locking version of address
    id_version: IdVersion,
    /// name of location
    name: String,
    /// street address
    street: String,
    /// postal code
    postal_code: String,
    /// city
    locality: String,
    /// optional region
    region: Option<String>,
    /// country: ISO code
    country: Option<CountryCode>,
}

impl ObjectIdVersion for PostalAddress {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

impl PostalAddress {
    pub fn new(id_version: IdVersion) -> PostalAddress {
        PostalAddress {
            id_version,
            ..Default::default()
        }
    }
    pub fn get_id(&self) -> Uuid {
        self.id_version.get_id()
    }
    pub fn get_version(&self) -> Option<u32> {
        self.id_version.get_version()
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get_street(&self) -> &str {
        self.street.as_str()
    }
    pub fn get_postal_code(&self) -> &str {
        self.postal_code.as_str()
    }
    pub fn get_locality(&self) -> &str {
        self.locality.as_str()
    }
    pub fn get_region(&self) -> Option<&str> {
        self.region.as_deref()
    }
    pub fn get_country(&self) -> Option<CountryCode> {
        self.country
    }

    pub fn set_id_version(&mut self, id_version: IdVersion) -> &mut Self {
        self.id_version = id_version;
        self
    }

    /// Sets the display name with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::PostalAddress;
    ///
    /// // Start from default.
    /// let mut addr = PostalAddress::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// addr.set_name("  Main   Campus  ");
    /// assert_eq!(addr.get_name(), "Main Campus");
    /// ```
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = normalize_ws(value);
        self
    }

    /// Sets the street with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::PostalAddress;
    ///
    /// // Start from default.
    /// let mut addr = PostalAddress::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// addr.set_street("  Muster   Straße   1  ");
    /// assert_eq!(addr.get_street(), "Muster Straße 1");
    /// ```
    pub fn set_street(&mut self, value: impl Into<String>) -> &mut Self {
        self.street = normalize_ws(value);
        self
    }

    /// Sets the postal code with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::PostalAddress;
    ///
    /// // Start from default.
    /// let mut addr = PostalAddress::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// addr.set_postal_code("  10115   ");
    /// assert_eq!(addr.get_postal_code(), "10115");
    /// ```
    pub fn set_postal_code(&mut self, value: impl Into<String>) -> &mut Self {
        self.postal_code = normalize_ws(value);
        self
    }

    /// Sets the locality (city) with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::PostalAddress;
    ///
    /// // Start from default.
    /// let mut addr = PostalAddress::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// addr.set_locality("  Berlin   Mitte  ");
    /// assert_eq!(addr.get_locality(), "Berlin Mitte");
    /// ```
    pub fn set_locality(&mut self, value: impl Into<String>) -> &mut Self {
        self.locality = normalize_ws(value);
        self
    }

    /// Sets the optional region with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    /// - converts empty/whitespace-only input to `None`
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::PostalAddress;
    ///
    /// // Start from default.
    /// let mut addr = PostalAddress::default();
    ///
    /// // 1) Regularize spacing (trim + collapse):
    /// addr.set_region("   BE   ");
    /// assert_eq!(addr.get_region(), Some("BE"));
    ///
    /// // 2) Whitespace-only becomes None:
    /// addr.set_region("   ");
    /// assert_eq!(addr.get_region(), None);
    /// ```
    pub fn set_region(&mut self, value: impl Into<String>) -> &mut Self {
        self.region = normalize_opt(Some(value));
        self
    }

    /// Sets the country code
    pub fn set_country(&mut self, value: Option<CountryCode>) -> &mut Self {
        self.country = value;
        self
    }

    pub fn validate(&self) -> ValidationResult<()> {
        let mut errs = ValidationErrors::new();
        let object_id = self.get_id();

        // Required fields (syntax-level)
        if self.name.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field("Name")
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.street.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field("Street")
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.postal_code.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field("PostalCode")
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.locality.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field("Locality")
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.country.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("Country")
                    .add_required()
                    .set_object_id(object_id)
                    .build(),
            );
        }

        // Example country-specific hint (non-blocking placeholder) for Germany
        if self.country == Some(CountryCode::DEU)
            && (self.postal_code.len() != 5
                || self.postal_code.chars().any(|c| !c.is_ascii_digit()))
        {
            errs.add(
                FieldError::builder()
                    .set_field("PostalCode")
                    .add_invalid_format()
                    .add_message("DE postal code must have 5 digits")
                    .set_object_id(object_id)
                    .build(),
            );
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

/// State for postal address operations
pub struct PostalAddressState {
    address: PostalAddress,
}

/// API of postal address
// switch state to postal address state to provide API for PostalAddress
impl<S> Core<S> {
    pub fn as_postal_address_state(&self) -> Core<PostalAddressState> {
        self.switch_state(PostalAddressState {
            address: PostalAddress::default(),
        })
    }
}

impl Core<PostalAddressState> {
    pub fn get(&self) -> &PostalAddress {
        &self.state.address
    }
    pub fn get_mut(&mut self) -> &mut PostalAddress {
        &mut self.state.address
    }
    pub async fn load(&mut self, id: Uuid) -> CoreResult<Option<&PostalAddress>> {
        if let Some(address) = self.database.get_postal_address(id).await? {
            self.state.address = address;
            self.state.address.validate()?;
            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }
    pub async fn save(&mut self) -> CoreResult<&PostalAddress> {
        // validate before save
        self.state.address.validate()?;
        // persist address
        self.state.address = self
            .database
            .save_postal_address(&self.state.address)
            .await?;
        // publish change of address to client registry
        let id = self.state.address.get_id();
        let version =
            self.state.address.get_version().expect(
                "expecting save_postal_address to return always an existing id and version",
            );
        let notice = CrTopic::Address(id);
        let msg = CrMsg::AddressUpdated { id, version };
        self.client_registry.publish(notice, msg).await?;
        Ok(self.get())
    }
    pub async fn list_addresses(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<PostalAddress>> {
        let list = self
            .database
            .list_postal_addresses(name_filter, limit)
            .await?;
        for addr in &list {
            addr.validate()?;
        }
        Ok(list)
    }
}

#[cfg(test)]
mod test_validate {
    use super::*;

    // Helper to build a *valid* baseline address we can then tweak per test.
    fn valid_addr() -> PostalAddress {
        let id_version = IdVersion::new(Uuid::new_v4(), Some(0));
        let mut pa = PostalAddress::new(id_version);
        pa.set_name("Main Campus")
            .set_street("Musterstraße 1")
            .set_postal_code("10115")
            .set_locality("Berlin")
            .set_region("BE")
            .set_country(Some(CountryCode::DEU));
        pa
    }

    // ── Required fields ──────────────────────────────────────────────────────

    #[test]
    fn given_valid_address_when_validate_then_ok() {
        let addr = valid_addr();
        let res = addr.validate();
        assert!(res.is_ok(), "a fully valid address should pass validation");
    }

    #[test]
    fn given_empty_name_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.name = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty name must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(err.get_field(), "Name", "should report the name field");
        assert_eq!(
            err.get_code(),
            "required",
            "should classify as 'required' violation"
        );
    }

    #[test]
    fn given_empty_street_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.street = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty street must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(err.get_field(), "Street", "should report the street field");
        assert_eq!(
            err.get_code(),
            "required",
            "should classify as 'required' violation"
        );
    }

    #[test]
    fn given_empty_postal_code_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.postal_code = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty postal code must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            "PostalCode",
            "should report the postal code field"
        );
        assert_eq!(
            err.get_code(),
            "required",
            "should classify as 'required' violation"
        );
    }

    #[test]
    fn given_empty_locality_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.locality = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty city/locality must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            "Locality",
            "should report the locality field"
        );
        assert_eq!(
            err.get_code(),
            "required",
            "should classify as 'required' violation"
        );
    }

    #[test]
    fn given_empty_country_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.country = None;

        let res = addr.validate();
        assert!(res.is_err(), "empty country must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            "Country",
            "should report the country field"
        );
        assert_eq!(
            err.get_code(),
            "required",
            "should classify as 'required' violation"
        );
    }

    #[test]
    fn given_multiple_required_fields_missing_when_validate_then_all_errors_are_collected() {
        let mut addr = valid_addr();
        addr.street = "".into();
        addr.postal_code = "".into();
        addr.locality = "".into();
        addr.country = None;

        let res = addr.validate();
        assert!(res.is_err(), "should fail with multiple missing fields");

        let errs = res.unwrap_err();
        assert_eq!(
            errs.errors.len(),
            4,
            "should report all four missing fields"
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), "Street"))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), "PostalCode"))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), "Locality"))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), "Country"))
        );
    }

    // ── Optional fields ──────────────────────────────────────────────────────

    #[test]
    fn given_missing_optional_fields_when_validate_then_ok() {
        let mut addr = valid_addr();
        addr.region = None;

        let res = addr.validate();
        assert!(res.is_ok(), "optional fields (region) may be None");
    }

    // ── Country-specific postal requirements (DE example) ────────────────────

    #[test]
    fn given_de_country_and_non_digit_postal_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.postal_code = "10A15".into();

        let res = addr.validate();
        assert!(res.is_err(), "DE postal code must be numeric");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            "PostalCode",
            "should report the postal code field"
        );
        assert_eq!(
            err.get_code(),
            "invalid_format",
            "should classify as 'invalid_format' violation"
        );
    }

    #[test]
    fn given_de_country_and_wrong_length_postal_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.postal_code = "1011".into();

        let res = addr.validate();
        assert!(res.is_err(), "DE postal code must have length 5");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            "PostalCode",
            "should report the postal code field"
        );
        assert_eq!(
            err.get_code(),
            "invalid_format",
            "should classify as 'invalid_format' violation"
        );
    }
}
