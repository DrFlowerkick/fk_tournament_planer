// @generated automatically by Diesel CLI.

diesel::table! {
    postal_addresses (id) {
        id -> Uuid,
        version -> Int8,
        name -> Citext,
        street -> Text,
        postal_code -> Text,
        locality -> Text,
        region -> Nullable<Text>,
        country -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
