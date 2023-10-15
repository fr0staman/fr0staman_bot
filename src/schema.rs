// @generated automatically by Diesel CLI.

diesel::table! {
    game (id) {
        id -> Integer,
        group_id -> Integer,
        mass -> Integer,
        date -> Date,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 64]
        f_name -> Varchar,
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
    inline_groups (id) {
        id -> Integer,
        chat_instance -> Bigint,
        invited_at -> Datetime,
    }
}

diesel::table! {
    inline_users (id) {
        id -> Integer,
        #[max_length = 64]
        f_name -> Varchar,
        weight -> Integer,
        date -> Date,
        #[max_length = 2]
        lang -> Varchar,
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
    }
}

diesel::joinable!(game -> groups (group_id));
diesel::joinable!(game -> users (uid));
diesel::joinable!(hryak_day -> inline_users_groups (iug_id));
diesel::joinable!(inline_users -> users (uid));
diesel::joinable!(inline_users_groups -> inline_groups (ig_id));
diesel::joinable!(inline_users_groups -> inline_users (iu_id));
diesel::joinable!(inline_voices -> users (uid));

diesel::allow_tables_to_appear_in_same_query!(
    game,
    groups,
    hryak_day,
    inline_groups,
    inline_users,
    inline_users_groups,
    inline_voices,
    users,
);
