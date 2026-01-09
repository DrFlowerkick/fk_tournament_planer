-- Enable required extensions (idempotent)
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- Main table for tournament base parameters
CREATE TABLE IF NOT EXISTS tournament_bases (
  id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Optimistic locking
  version          bigint      NOT NULL DEFAULT 0,

  -- A human-readable name for the tournament and foreign key to the sport type
  name             citext      NOT NULL,
  sport_id         uuid        NOT NULL,

  -- Size of the tournament
  num_entrants     integer     NOT NULL,

  -- Configuration and state stored as JSONB
  t_type           jsonb       NOT NULL,  -- TournamentType
  mode             jsonb       NOT NULL,  -- TournamentMode
  state            jsonb       NOT NULL,  -- TournamentState

  -- Timestamps
  created_at       timestamptz NOT NULL DEFAULT now(),
  updated_at       timestamptz NOT NULL DEFAULT now(),

  -- Constraints
  CONSTRAINT version_non_negative CHECK (version >= 0)
);

-- Enforce uniqueness of tournament names per sport
CREATE UNIQUE INDEX IF NOT EXISTS uniq_tournament_bases_name_per_sport
  ON tournament_bases (sport_id, name);

-- Re-use the existing updated_at maintenance trigger function
-- (assuming it's globally available from the previous migration)
DROP TRIGGER IF EXISTS set_timestamp_tournament_bases ON tournament_bases;
CREATE TRIGGER set_timestamp_tournament_bases
BEFORE UPDATE ON tournament_bases
FOR EACH ROW
EXECUTE FUNCTION trg_set_timestamp();
