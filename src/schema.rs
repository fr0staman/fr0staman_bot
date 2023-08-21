// @generated automatically by Diesel CLI.

diesel::table! {
    counter (id) {
        id -> Integer,
        count -> Integer,
    }
}

diesel::table! {
    game (id) {
        id -> Integer,
        user_id -> Unsigned<Bigint>,
        group_id -> Integer,
        mass -> Integer,
        date -> Date,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 64]
        f_name -> Varchar,
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
    }
}

diesel::table! {
    inline_users (id) {
        id -> Integer,
        user_id -> Unsigned<Bigint>,
        #[max_length = 64]
        f_name -> Varchar,
        weight -> Integer,
        date -> Date,
        #[max_length = 2]
        lang -> Varchar,
        win -> Unsigned<Smallint>,
        rout -> Unsigned<Smallint>,
        status -> Tinyint,
        #[max_length = 64]
        name -> Varchar,
        gifted -> Bool,
        #[max_length = 17]
        flag -> Varchar,
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
        user_id -> Unsigned<Bigint>,
        #[max_length = 64]
        caption -> Varchar,
        status -> Smallint,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Smallint>,
        user_id -> Unsigned<Bigint>,
        status -> Tinyint,
    }
}

diesel::joinable!(game -> groups (group_id));
diesel::joinable!(hryak_day -> inline_users_groups (iug_id));
diesel::joinable!(inline_users_groups -> inline_groups (ig_id));
diesel::joinable!(inline_users_groups -> inline_users (iu_id));

diesel::allow_tables_to_appear_in_same_query!(
    counter,
    game,
    groups,
    hryak_day,
    inline_groups,
    inline_users,
    inline_users_groups,
    inline_voices,
    users,
);
