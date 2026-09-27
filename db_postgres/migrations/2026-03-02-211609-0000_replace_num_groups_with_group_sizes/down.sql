-- This file should undo anything in `up.sql`
-- 1. Restore old column
ALTER TABLE stages 
ADD COLUMN num_groups INTEGER NOT NULL DEFAULT 1;

-- 2. Revert data:
-- We use the length of the array (cardinality) as the number of groups.
UPDATE stages 
SET num_groups = cardinality(group_sizes);

-- 3. Remove new column
ALTER TABLE stages 
DROP COLUMN group_sizes;