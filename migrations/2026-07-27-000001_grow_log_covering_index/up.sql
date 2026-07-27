-- Every read of grow_log is "the feeds of one pig, newest first": the
-- achievement window on each /grow, and the 14-day chart queries for /my and
-- /top. The key columns already served those lookups, but each returned row
-- still cost a random heap fetch, because one pig's rows are spread across
-- years of inserts from thousands of other pigs.
--
-- Carrying weight_change/current_weight as index payload makes those reads
-- index-only scans. GrowLog does not select `id`, which is what allows it.
CREATE INDEX CONCURRENTLY IF NOT EXISTS grow_log_game_id_created_at_covering_idx
    ON grow_log (game_id, created_at DESC)
    INCLUDE (weight_change, current_weight);

-- Superseded: identical key columns, no payload. Keeping both would double
-- the index maintenance on every insert for no read benefit.
DROP INDEX CONCURRENTLY IF EXISTS grow_log_game_id_created_at_idx;
