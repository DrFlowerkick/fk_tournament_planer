-- This file should undo anything in `up.sql`
-- Drop the partial unique index first (it would be dropped implicitly with the table,
-- but dropping explicitly is safe and keeps intent clear).
DROP INDEX IF EXISTS uniq_tournament_bases_name_per_sport;

-- Drop the table
DROP TABLE IF EXISTS tournament_bases;

-- Extensions are typically kept installed for other objects that may rely on them.
-- Uncomment if you really want to remove them:
-- DROP EXTENSION IF EXISTS citext;
-- DROP EXTENSION IF EXISTS pgcrypto;