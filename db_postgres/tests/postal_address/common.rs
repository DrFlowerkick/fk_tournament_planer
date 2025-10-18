use app_core::{PostalAddress, utils::id_version::IdVersion};

/// Build a valid "new" PostalAddress with deterministic fields.
/// NOTE: `name` and `region` are optional at the DB level; we fill everything
/// to avoid validation pitfalls.
pub fn make_new_address(label: &str) -> PostalAddress {
    // Construct as "new": DB will assign id and initialize version=0
    let mut pa = PostalAddress::new(IdVersion::New);

    // Use deterministic but distinct content to ease debugging
    pa.set_name(format!("Name {label}"))
        .set_street(format!("{label} Street 1"))
        .set_postal_code("12345")
        .set_locality("Berlin")
        .set_region("BE")
        .set_country("DE");

    pa
}

/// Mutate the address to a second version (change some fields).
pub fn mutate_address_v2(mut pa: PostalAddress) -> PostalAddress {
    pa.set_street("Changed Street 99")
        .set_postal_code("54321")
        .set_locality("Potsdam");
    pa
}
