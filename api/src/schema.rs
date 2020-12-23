table! {
    invitations (id) {
        id -> Uuid,
        email -> Varchar,
        expires_at -> Timestamptz,
        forgot_pw -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        hash -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    invitations,
    users,
);
