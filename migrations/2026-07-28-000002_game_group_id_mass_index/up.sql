-- (group_id, mass DESC) serves "the pigs of one group, heaviest first": /top's
-- ordered page, the per-group counts, and the correlated pig-count that decides
-- whether a chat is active. Carrying mass also makes /top's ORDER BY free.
CREATE INDEX CONCURRENTLY IF NOT EXISTS game_group_id_mass_idx
    ON game (group_id, mass DESC);
