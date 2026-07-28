-- How every incoming update is resolved to a row (maybe_get_or_insert_user runs
-- before any handler), so this was a sequential scan per update.
--
-- Deliberately not UNIQUE, though the column holds one row per Telegram id: the
-- table came over from MySQL, and CREATE UNIQUE INDEX CONCURRENTLY leaves an
-- INVALID index behind if duplicates exist. Check before promoting it:
--   SELECT user_id FROM users GROUP BY user_id HAVING COUNT(*) > 1
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_user_id_idx
    ON users (user_id);
