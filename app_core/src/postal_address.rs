// data types for postal addresses

use crate::{
    Core, CrPushNotice, CrUpdateMeta, DbResult,
    utils::{id_version::IdVersion, normalize::*, validation::*},
};
use displaydoc::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display)]
pub enum PaValidationField {
    /// street
    Street,
    /// postal code
    PostalCode,
    /// locality
    Locality,
    /// country
    Country,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PostalAddress {
    /// id and optimistic locking version of address
    id_version: IdVersion,
    /// name of location
    name: Option<String>,
    /// street address
    street: String,
    /// postal code
    postal_code: String,
    /// city
    locality: String,
    /// optional region
    region: Option<String>,
    /// country: ISO name or code
    country: String,
}

impl Default for PostalAddress {
    fn default() -> Self {
        PostalAddress {
            id_version: IdVersion::New,
            name: None,
            street: "".into(),
            postal_code: "".into(),
            locality: "".into(),
            region: None,
            country: "".into(),
        }
    }
}

impl PostalAddress {
    pub fn new(id_version: IdVersion) -> PostalAddress {
        PostalAddress {
            id_version,
            ..Default::default()
        }
    }
    pub fn get_id(&self) -> Option<Uuid> {
        self.id_version.get_id()
    }
    pub fn get_version(&self) -> Option<u32> {
        self.id_version.get_version()
    }
    pub fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
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
    pub fn get_country(&self) -> &str {
        self.country.as_str()
    }

    pub fn set_id_version(&mut self, id_version: IdVersion) -> &mut Self {
        self.id_version = id_version;
        self
    }

    /// Sets the optional display name with normalization:
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
    /// addr.set_name("  Main   Campus  ");
    /// assert_eq!(addr.get_name(), Some("Main Campus"));
    ///
    /// // 2) Whitespace-only becomes None:
    /// addr.set_name("   ");
    /// assert_eq!(addr.get_name(), None);
    /// ```
    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = normalize_opt(Some(value));
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

    /// Sets the country code with normalization:
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    /// - converts code to upper case
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
    /// addr.set_country("  De  ");
    /// assert_eq!(addr.get_country(), "DE");
    /// ```
    pub fn set_country(&mut self, value: impl Into<String>) -> &mut Self {
        self.country = normalize_ws(value).to_uppercase();
        self
    }

    pub fn validate(&self) -> Result<(), ValidationErrors<PaValidationField>> {
        let mut errs = ValidationErrors::new();

        // Required fields (syntax-level)
        if self.street.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(PaValidationField::Street)
                    .add_required()
                    .build(),
            );
        }
        if self.postal_code.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(PaValidationField::PostalCode)
                    .add_required()
                    .build(),
            );
        }
        if self.locality.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(PaValidationField::Locality)
                    .add_required()
                    .build(),
            );
        }
        if self.country.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(PaValidationField::Country)
                    .add_required()
                    .build(),
            );
        }

        // Example country-specific hint (non-blocking placeholder):
        if self.country == "DE"
            && (self.postal_code.len() != 5
                || self.postal_code.chars().any(|c| !c.is_ascii_digit()))
        {
            errs.add(
                FieldError::builder()
                    .set_field(PaValidationField::PostalCode)
                    .add_invalid_format()
                    .add_message("DE postal code must have 5 digits")
                    .build(),
            );
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

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
    pub async fn load(&mut self, id: Uuid) -> DbResult<Option<&PostalAddress>> {
        if let Some(address) = self.database.get_postal_address(id).await? {
            self.state.address = address;
            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }
    pub async fn save(&mut self) -> DbResult<&PostalAddress> {
        self.state.address = self
            .database
            .save_postal_address(&self.state.address)
            .await?;
        // publish change of address to client registry
        let notice =
            CrPushNotice::AddressUpdated {
                id: self
                    .state
                    .address
                    .get_id()
                    .expect("expecting save_postal_address to return always an existing uuid"),
                meta: CrUpdateMeta {
                    version: self.state.address.get_version().expect(
                        "expecting save_postal_address to return always an existing version",
                    ),
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

#[cfg(test)]
mod test_validate {
    use super::*;

    // Helper to build a *valid* baseline address we can then tweak per test.
    fn valid_addr() -> PostalAddress {
        let id_version = IdVersion::new(Uuid::new_v4(), 0);
        let mut pa = PostalAddress::new(id_version);
        pa.set_name("Main Campus")
            .set_street("Musterstraße 1")
            .set_postal_code("10115")
            .set_locality("Berlin")
            .set_region("BE")
            .set_country("DE");
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
    fn given_empty_street_when_validate_then_err() {
        let mut addr = valid_addr();
        addr.street = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty street must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            &PaValidationField::Street,
            "should report the street field"
        );
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
            &PaValidationField::PostalCode,
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
            &PaValidationField::Locality,
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
        addr.country = "".into();

        let res = addr.validate();
        assert!(res.is_err(), "empty country must be rejected");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            &PaValidationField::Country,
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
        addr.country = "".into();

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
                .any(|e| matches!(e.get_field(), PaValidationField::Street))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), PaValidationField::PostalCode))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), PaValidationField::Locality))
        );
        assert!(
            errs.errors
                .iter()
                .any(|e| matches!(e.get_field(), PaValidationField::Country))
        );
    }

    // ── Optional fields ──────────────────────────────────────────────────────

    #[test]
    fn given_missing_optional_fields_when_validate_then_ok() {
        let mut addr = valid_addr();
        addr.name = None;
        addr.region = None;

        let res = addr.validate();
        assert!(res.is_ok(), "optional fields (name, region) may be None");
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
            &PaValidationField::PostalCode,
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
        addr.postal_code = "1011".into(); // 4 digits

        let res = addr.validate();
        assert!(res.is_err(), "DE postal code must have length 5");

        let errs = res.unwrap_err();
        let err = errs.errors.first().unwrap();
        assert_eq!(
            err.get_field(),
            &PaValidationField::PostalCode,
            "should report the postal code field"
        );
        assert_eq!(
            err.get_code(),
            "invalid_format",
            "should classify as 'invalid_format' violation"
        );
    }
}
