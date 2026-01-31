diesel::table! {
    customers (id) {
        id -> Text,
        schema_name -> Text,
        created_at -> Nullable<Timestamptz>,
    }
}
