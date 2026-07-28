-- Read on every /grow (the achievement check), /my and /achievements. No
-- INCLUDE payload: the query selects the whole row including `id`, and a pig
-- only ever has a few dozen achievements, so the heap fetches are cheap.
CREATE INDEX CONCURRENTLY IF NOT EXISTS achievements_users_game_id_idx
    ON achievements_users (game_id);
