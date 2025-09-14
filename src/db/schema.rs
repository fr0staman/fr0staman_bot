// @generated automatically by Diesel CLI.

diesel::table! {
    achievements_users (id) {
        id -> Unsigned<Integer>,
        game_id -> Integer,
        created_at -> Datetime,
        code -> Unsigned<Smallint>,
    }
}

diesel::table! {
    game (id) {
        id -> Integer,
        group_id -> Integer,
        mass -> Integer,
        date -> Date,
        #[max_length = 64]
        name -> Varchar,
        uid -> Unsigned<Integer>,
    }
}

diesel::table! {
    groups (id) {
        id -> Integer,
        chat_id -> Bigint,
        date -> Datetime,
        settings -> Tinyint,
        top10_setting -> Integer,
        #[max_length = 2]
        lang -> Nullable<Varchar>,
        active -> Bool,
        ig_id -> Nullable<Integer>,
        #[max_length = 64]
        username -> Nullable<Varchar>,
        #[max_length = 128]
        title -> Varchar,
    }
}

diesel::table! {
    grow_log (id) {
        id -> Unsigned<Integer>,
        game_id -> Integer,
        created_at -> Datetime,
        weight_change -> Integer,
        current_weight -> Unsigned<Integer>,
    }
}

diesel::table! {
    hryak_day (id) {
        id -> Integer,
        iug_id -> Integer,
        date -> Date,
    }
}

diesel::table! {
    inline_gifs (id) {
        id -> Smallint,
        #[max_length = 128]
        file_id -> Varchar,
        status -> Smallint,
        uid -> Unsigned<Integer>,
        #[max_length = 64]
        file_unique_id -> Varchar,
    }
}

diesel::table! {
    inline_groups (id) {
        id -> Integer,
        chat_instance -> Bigint,
        invited_at -> Datetime,
    }
}

diesel::table! {
    inline_users (id) {
        id -> Integer,
        weight -> Integer,
        date -> Date,
        win -> Unsigned<Smallint>,
        rout -> Unsigned<Smallint>,
        #[max_length = 64]
        name -> Varchar,
        gifted -> Bool,
        #[max_length = 17]
        flag -> Varchar,
        uid -> Unsigned<Integer>,
    }
}

diesel::table! {
    inline_users_groups (id) {
        id -> Integer,
        iu_id -> Integer,
        ig_id -> Integer,
    }
}

diesel::table! {
    inline_voices (id) {
        id -> Smallint,
        #[max_length = 128]
        url -> Varchar,
        #[max_length = 64]
        caption -> Varchar,
        status -> Smallint,
        uid -> Unsigned<Integer>,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Bigint>,
        started -> Bool,
        subscribed -> Bool,
        supported -> Bool,
        banned -> Bool,
        created_at -> Datetime,
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
