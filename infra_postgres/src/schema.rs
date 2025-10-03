// @generated automatically by Diesel CLI.

diesel::table! {
    postal_addresses (id) {
        id -> Uuid,
        version -> Int8,
        name -> Nullable<Citext>,
        street_address -> Text,
        postal_code -> Text,
        address_locality -> Text,
        address_region -> Nullable<Text>,
        address_country -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
