// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        block_id -> Integer,
        content -> Text,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    roles (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        first_name -> Text,
        last_name -> Text,
        role_id -> Nullable<Integer>,
        apartment -> Nullable<Text>,
        block_id -> Nullable<Integer>,
        password -> Text,
        photo -> Nullable<Binary>,
    }
}

diesel::joinable!(messages -> blocks (block_id));
diesel::joinable!(messages -> users (user_id));
diesel::joinable!(users -> blocks (block_id));
diesel::joinable!(users -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    messages,
    roles,
    users,
);
