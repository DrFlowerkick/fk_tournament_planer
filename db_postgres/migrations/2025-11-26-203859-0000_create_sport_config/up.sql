-- Enable required extensions (idempotent)
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- Main table for sport configurations
CREATE TABLE IF NOT EXISTS sport_configs (
  id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Optimistic locking
  version          bigint      NOT NULL DEFAULT 0,

  -- Foreign key to the sport type and a human-readable name for the config
  sport_id         uuid        NOT NULL,
  name             citext      NOT NULL,

  -- The actual configuration data
  config           jsonb       NOT NULL,

  -- Timestamps
  created_at       timestamptz NOT NULL DEFAULT now(),
  updated_at       timestamptz NOT NULL DEFAULT now(),

  -- Constraints
  CONSTRAINT version_non_negative CHECK (version >= 0)
);

-- Enforce uniqueness of configuration names per sport
CREATE UNIQUE INDEX IF NOT EXISTS uniq_sport_configs_name_per_sport
  ON sport_configs (sport_id, name);

-- Re-use the existing updated_at maintenance trigger function
-- (assuming it's globally available from the previous migration)
DROP TRIGGER IF EXISTS set_timestamp_sport_configs ON sport_configs;
CREATE TRIGGER set_timestamp_sport_configs
BEFORE UPDATE ON sport_configs
FOR EACH ROW
EXECUTE FUNCTION trg_set_timestamp();