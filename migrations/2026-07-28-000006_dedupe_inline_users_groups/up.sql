-- Clears the way for the unique index in the next migration.
--
-- `get_or_create_iug` is a select-then-insert with no constraint behind it,
-- so two concurrent inline queries from the same user in the same chat could
-- both miss and both insert. Any duplicates already in the table have to go
-- before a unique index can be built.

-- Day-pig history points at a specific link row, so repoint it at the row
-- that survives before deleting the others. `hryak_day` has no unique
-- constraint on (iug_id, date), so this cannot collide.
UPDATE hryak_day
SET iug_id = keep.id
FROM inline_users_groups dup
JOIN (
    SELECT iu_id, ig_id, MIN(id) AS id
    FROM inline_users_groups
    GROUP BY iu_id, ig_id
) keep ON keep.iu_id = dup.iu_id AND keep.ig_id = dup.ig_id
WHERE hryak_day.iug_id = dup.id
  AND dup.id <> keep.id;

-- Keep the lowest id of each (iu_id, ig_id) pair.
DELETE FROM inline_users_groups a
USING inline_users_groups b
WHERE a.iu_id = b.iu_id
  AND a.ig_id = b.ig_id
  AND a.id > b.id;
