diesel::table! {
  users (id) {
    id -> BigInt,
    name -> Text,
    division_name -> SmallInt,
    division_rank -> SmallInt,
  }
}

diesel::table! {
  user_stats (user_id) {
    user_id -> BigInt,
    stats -> Jsonb,
  }
}

diesel::table! {
  ugcs (id) {
    id -> Uuid,
    user_id -> BigInt,
    name -> Text,
    created_at -> Timestamp,
    updated_at -> Timestamp,
    published -> Bool,
    type_id -> SmallInt,
    transform -> Jsonb,
    map_position -> Nullable<Jsonb>,
    // TODO: figure out why it was needed in the original
    // and probably remove it
    teleport_transform -> Nullable<Jsonb>,
  }
}

diesel::table! {
    ugc_bookmarks (user_id, ugc_id) {
        user_id -> BigInt,
        ugc_id -> Uuid,
        bookmark_time -> Timestamp,
    }
}

diesel::table! {
    challenge_bookmarks (user_id, challenge_id) {
        user_id -> BigInt,
        challenge_id -> Uuid,
        bookmark_time -> Timestamp,
    }
}

diesel::table! {
    kit_unlocks (user_id, kit_id) {
        user_id -> BigInt,
        kit_id -> Uuid,
        opened -> Bool,
    }
}

diesel::joinable!(user_stats -> users (user_id));
diesel::joinable!(ugcs -> users (user_id));
diesel::joinable!(ugc_bookmarks -> users (user_id));
diesel::joinable!(ugc_bookmarks -> ugcs (ugc_id));
diesel::joinable!(challenge_bookmarks -> users (user_id));
diesel::joinable!(kit_unlocks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    user_stats,
    ugcs,
    ugc_bookmarks,
    challenge_bookmarks,
    kit_unlocks,
);
