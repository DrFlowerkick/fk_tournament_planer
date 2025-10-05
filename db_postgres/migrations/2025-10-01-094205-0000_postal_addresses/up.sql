-- Enable required extensions (idempotent)
CREATE EXTENSION IF NOT EXISTS pgcrypto;  -- gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS citext;    -- case-insensitive text

-- Main table
CREATE TABLE IF NOT EXISTS postal_addresses (
  id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Optimistic locking
  version          bigint      NOT NULL DEFAULT 0,

  -- Optional, case-insensitive name of the location (NULL = no name)
  name             citext,

  -- Address fields
  street_address   text        NOT NULL,
  postal_code      text        NOT NULL,
  address_locality text        NOT NULL,
  address_region   text,
  address_country  text        NOT NULL,

  -- Timestamps
  created_at       timestamptz NOT NULL DEFAULT now(),
  updated_at       timestamptz NOT NULL DEFAULT now(),

  -- Constraints
  CONSTRAINT name_not_blank CHECK (name IS NULL OR length(btrim(name::text)) > 0),
  CONSTRAINT version_non_negative CHECK (version >= 0)
);

-- Enforce uniqueness of (name, postal_code, address_locality) only when name is present
CREATE UNIQUE INDEX IF NOT EXISTS uniq_postal_addresses_name_per_city_zip
  ON postal_addresses (name, postal_code, address_locality)
  WHERE name IS NOT NULL;

-- updated_at maintenance trigger (keine Automatik für version!)
CREATE OR REPLACE FUNCTION trg_set_timestamp()
RETURNS trigger AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_timestamp_postal_addresses ON postal_addresses;
CREATE TRIGGER set_timestamp_postal_addresses
BEFORE UPDATE ON postal_addresses
FOR EACH ROW
EXECUTE FUNCTION trg_set_timestamp();
