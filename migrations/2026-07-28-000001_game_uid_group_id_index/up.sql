-- `game` carried no index but its primary key, so every lookup starting from a
-- user or a group scanned the whole table.
--
-- (uid, group_id) serves "this user's pigs": the achievement check that counts
-- a user's active chats, and /my's biggest-pig lookup. Both read only these two
-- columns, so the scan stays index-only.
CREATE INDEX CONCURRENTLY IF NOT EXISTS game_uid_group_id_idx
    ON game (uid, group_id);
