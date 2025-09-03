// @generated automatically by Diesel CLI.

diesel::table! {
    challenge_bookmarks (user_id, challenge_id) {
        user_id -> Int8,
        challenge_id -> Uuid,
        bookmark_time -> Timestamp,
    }
}

diesel::table! {
    kit_unlocks (user_id, kit_id) {
        user_id -> Int8,
        kit_id -> Uuid,
        opened -> Bool,
    }
}

diesel::table! {
    ugc_bookmarks (user_id, ugc_id) {
        user_id -> Int8,
        ugc_id -> Uuid,
        bookmark_time -> Timestamp,
    }
}

diesel::table! {
    ugcs (id) {
        id -> Uuid,
        user_id -> Int8,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        published -> Bool,
        type_id -> Int2,
        transform -> Jsonb,
        map_position -> Nullable<Jsonb>,
        teleport_transform -> Nullable<Jsonb>,
    }
}

diesel::table! {
    user_stats (user_id) {
        user_id -> Int8,
        stats -> Jsonb,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        name -> Text,
        division_name -> Int2,
        division_rank -> Int2,
    }
}

diesel::joinable!(challenge_bookmarks -> users (user_id));
diesel::joinable!(kit_unlocks -> users (user_id));
diesel::joinable!(ugc_bookmarks -> ugcs (ugc_id));
diesel::joinable!(ugc_bookmarks -> users (user_id));
diesel::joinable!(ugcs -> users (user_id));
diesel::joinable!(user_stats -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    challenge_bookmarks,
    kit_unlocks,
    ugc_bookmarks,
    ugcs,
    user_stats,
    users,
);