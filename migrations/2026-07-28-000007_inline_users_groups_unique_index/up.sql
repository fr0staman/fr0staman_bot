-- Makes `get_or_create_iug`'s select-then-insert race-safe rather than merely
-- usually-safe: a losing concurrent insert now fails instead of creating a
-- second link row. Also replaces the plain lookup index for
-- `get_iug_by_ids`, which filters on exactly these two columns.
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS
    inline_users_groups_iu_id_ig_id_key
    ON inline_users_groups (iu_id, ig_id);
