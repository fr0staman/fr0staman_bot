-- Superseded by grow_log_game_id_created_at_covering_idx: identical key
-- columns, no payload. Keeping both would double the index maintenance on
-- every insert for no read benefit.
DROP INDEX CONCURRENTLY IF EXISTS grow_log_game_id_created_at_idx;
