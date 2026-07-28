-- The three remaining unindexed lookups the bot performs constantly.
--
-- users.user_id and groups.chat_id are how every incoming update is resolved
-- to a row (maybe_get_or_insert_user / _chat run before any handler), so these
-- two were a sequential scan per update.
--
-- Deliberately not UNIQUE, even though both columns hold one row per Telegram
-- id: the tables came over from MySQL, and CREATE UNIQUE INDEX CONCURRENTLY
-- leaves an INVALID index behind if duplicates exist. To promote them later,
-- check first with:
--   SELECT user_id FROM users GROUP BY user_id HAVING COUNT(*) > 1;
--   SELECT chat_id FROM groups GROUP BY chat_id HAVING COUNT(*) > 1;
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_user_id_idx
    ON users (user_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS groups_chat_id_idx
    ON groups (chat_id);

-- Read on every /grow (the achievement check), /my and /achievements. No
-- INCLUDE payload: the query selects the whole row including `id`, and a pig
-- only ever has a few dozen achievements, so the heap fetches are cheap.
CREATE INDEX CONCURRENTLY IF NOT EXISTS achievements_users_game_id_idx
    ON achievements_users (game_id);
