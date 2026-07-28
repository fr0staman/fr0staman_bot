-- Same story as users.user_id: maybe_get_or_insert_chat resolves every group
-- update through this column. Not UNIQUE for the same reason; check with
--   SELECT chat_id FROM groups GROUP BY chat_id HAVING COUNT(*) > 1
CREATE INDEX CONCURRENTLY IF NOT EXISTS groups_chat_id_idx
    ON groups (chat_id);
