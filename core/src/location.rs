// data types for postal addresses

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PostalAddress {
    /// id of address
    id: Uuid,
    /// street address
    street_address: String,
    /// postal code
    postal_code: String,
    /// city
    address_locality: String,
    /// optional region
    address_region: Option<String>,
    /// country: ISO name or code
    address_country: String,
}
