// @generated automatically by Diesel CLI.

diesel::table! {
    products (id) {
        id -> Nullable<Integer>,
        name -> Text,
        price -> Double,
        description -> Nullable<Text>,
        image -> Nullable<Binary>,
    }
}
