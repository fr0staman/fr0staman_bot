-- `game` carried no index but its primary key, so every lookup that starts
-- from a user or a group had to scan the whole table.
--
-- (uid, group_id) serves "this user's pigs": the achievement check that counts
-- a user's active chats, and /my's biggest-pig lookup. Both only read those two
-- columns, so the scan stays index-only.
CREATE INDEX CONCURRENTLY IF NOT EXISTS game_uid_group_id_idx
    ON game (uid, group_id);

-- (group_id, mass DESC) serves "the pigs of one group, heaviest first": /top's
-- ordered page, the per-group counts, and the correlated pig-count that decides
-- whether a chat is active. The mass column also makes /top's ORDER BY free.
CREATE INDEX CONCURRENTLY IF NOT EXISTS game_group_id_mass_idx
    ON game (group_id, mass DESC);
