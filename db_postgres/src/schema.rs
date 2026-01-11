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

diesel::table! {
    sport_configs (id) {
        id -> Uuid,
        version -> Int8,
        sport_id -> Uuid,
        name -> Citext,
        config -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    tournament_bases (id) {
        id -> Uuid,
        version -> Int8,
        name -> Citext,
        sport_id -> Uuid,
        num_entrants -> Int4,
        t_type -> Jsonb,
        mode -> Jsonb,
        state -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(postal_addresses, sport_configs, tournament_bases,);
