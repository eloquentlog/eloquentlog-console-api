table! {
    messages (id) {
        id -> Int8,
        code -> Nullable<Bpchar>,
        lang -> Bpchar,
        level -> Log_level,
        format -> Log_format,
        title -> Varchar,
        content -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
