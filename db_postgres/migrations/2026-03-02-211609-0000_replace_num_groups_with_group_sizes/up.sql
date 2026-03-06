-- Your SQL goes here
-- 1. Add new column (Postgres Array for Vec<i32>)
ALTER TABLE stages 
ADD COLUMN group_sizes INTEGER[] NOT NULL DEFAULT '{}';

-- 2. Migrate data: 
-- We create an array filled with 0s, the length matches the old 'num_groups'.
-- array_fill(value, ARRAY[length]) is the Postgres function used here.
UPDATE stages 
SET group_sizes = array_fill(0, ARRAY[num_groups])
WHERE num_groups > 0;

-- 3. Remove old column
ALTER TABLE stages 
DROP COLUMN num_groups;