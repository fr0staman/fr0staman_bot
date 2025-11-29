// @generated automatically by Diesel CLI.

diesel::table! {
    achievements_users (id) {
        id -> Int4,
        game_id -> Int4,
        created_at -> Timestamp,
        code -> Int2,
    }
}

diesel::table! {
    game (id) {
        id -> Int4,
        group_id -> Int4,
        mass -> Int4,
        date -> Date,
        #[max_length = 64]
        name -> Varchar,
        uid -> Int4,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        chat_id -> Int8,
        date -> Timestamp,
        settings -> Int2,
        top10_setting -> Int4,
        #[max_length = 2]
        lang -> Nullable<Varchar>,
        active -> Bool,
        ig_id -> Nullable<Int4>,
        #[max_length = 64]
        username -> Nullable<Varchar>,
        #[max_length = 128]
        title -> Varchar,
    }
}

diesel::table! {
    grow_log (id) {
        id -> Int4,
        game_id -> Int4,
        created_at -> Timestamp,
        weight_change -> Int4,
        current_weight -> Int4,
    }
}

diesel::table! {
    hryak_day (id) {
        id -> Int4,
        iug_id -> Int4,
        date -> Date,
    }
}

diesel::table! {
    inline_gifs (id) {
        id -> Int2,
        #[max_length = 128]
        file_id -> Varchar,
        status -> Int2,
        uid -> Int4,
        #[max_length = 64]
        file_unique_id -> Varchar,
    }
}

diesel::table! {
    inline_groups (id) {
        id -> Int4,
        chat_instance -> Int8,
        invited_at -> Timestamp,
    }
}

diesel::table! {
    inline_users (id) {
        id -> Int4,
        weight -> Int4,
        date -> Date,
        win -> Int4,
        rout -> Int4,
        #[max_length = 64]
        name -> Varchar,
        gifted -> Bool,
        #[max_length = 17]
        flag -> Varchar,
        uid -> Int4,
    }
}

diesel::table! {
    inline_users_groups (id) {
        id -> Int4,
        iu_id -> Int4,
        ig_id -> Int4,
    }
}

diesel::table! {
    inline_voices (id) {
        id -> Int2,
        #[max_length = 128]
        url -> Varchar,
        #[max_length = 64]
        caption -> Varchar,
        status -> Int2,
        uid -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        user_id -> Int8,
        started -> Bool,
        subscribed -> Bool,
        supported -> Bool,
        banned -> Bool,
        created_at -> Timestamp,
        #[max_length = 2]
        lang -> Nullable<Varchar>,
        #[max_length = 64]
        username -> Nullable<Varchar>,
        #[max_length = 64]
        last_name -> Nullable<Varchar>,
        #[max_length = 64]
        first_name -> Varchar,
    }
}

diesel::joinable!(achievements_users -> game (game_id));
diesel::joinable!(game -> groups (group_id));
diesel::joinable!(game -> users (uid));
diesel::joinable!(groups -> inline_groups (ig_id));
diesel::joinable!(grow_log -> game (game_id));
diesel::joinable!(hryak_day -> inline_users_groups (iug_id));
diesel::joinable!(inline_gifs -> users (uid));
diesel::joinable!(inline_users -> users (uid));
diesel::joinable!(inline_users_groups -> inline_groups (ig_id));
diesel::joinable!(inline_users_groups -> inline_users (iu_id));
diesel::joinable!(inline_voices -> users (uid));

diesel::allow_tables_to_appear_in_same_query!(
    achievements_users,
    game,
    groups,
    grow_log,
    hryak_day,
    inline_gifs,
    inline_groups,
    inline_users,
    inline_users_groups,
    inline_voices,
    users,
);
