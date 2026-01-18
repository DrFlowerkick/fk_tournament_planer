-- Enable required extensions (idempotent)
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Main table for tournament stages
CREATE TABLE IF NOT EXISTS stages (
  id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),

  -- Optimistic locking
  version          bigint      NOT NULL DEFAULT 0,

  -- Foreign key to the tournament
  tournament_id    uuid        NOT NULL,

  -- Scheduled stage number in tournament (0, 1, 2...)
  number           integer     NOT NULL,

  -- Configuration
  num_groups       integer     NOT NULL DEFAULT 1,

  -- Timestamps
  created_at       timestamptz NOT NULL DEFAULT now(),
  updated_at       timestamptz NOT NULL DEFAULT now(),

  -- Constraints
  CONSTRAINT version_non_negative CHECK (version >= 0),
  CONSTRAINT number_non_negative CHECK (number >= 0),
  CONSTRAINT num_groups_positive CHECK (num_groups > 0),
  
  -- Foreign Key Constraint
  CONSTRAINT fk_tournament
    FOREIGN KEY(tournament_id) 
    REFERENCES tournament_bases(id)
    ON DELETE CASCADE
);

-- Enforce uniqueness of stage number per tournament
-- (Equivalent to name per sport in tournament_bases)
CREATE UNIQUE INDEX IF NOT EXISTS uniq_stages_number_per_tournament
  ON stages (tournament_id, number);

-- Re-use the existing updated_at maintenance trigger function
DROP TRIGGER IF EXISTS set_timestamp_stages ON stages;
CREATE TRIGGER set_timestamp_stages
BEFORE UPDATE ON stages
FOR EACH ROW
EXECUTE FUNCTION trg_set_timestamp();