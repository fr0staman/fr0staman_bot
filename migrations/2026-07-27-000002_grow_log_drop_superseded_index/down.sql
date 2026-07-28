CREATE INDEX CONCURRENTLY IF NOT EXISTS grow_log_game_id_created_at_idx
    ON grow_log (game_id, created_at DESC);
