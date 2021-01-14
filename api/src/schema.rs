table! {
    arrows (source, target) {
        source -> Int4,
        target -> Int4,
    }
}

table! {
    invitations (id) {
        id -> Uuid,
        email -> Varchar,
        expires_at -> Timestamptz,
        forgot_pw -> Bool,
        tz -> Varchar,
    }
}

table! {
    permissions (subject, object) {
        subject -> Int4,
        object -> Int4,
        edit -> Bool,
    }
}

table! {
    tasks (id) {
        id -> Int4,
        title -> Varchar,
        assign -> Int4,
        is_archived -> Bool,
        is_starred -> Bool,
        startable -> Nullable<Timestamptz>,
        deadline -> Nullable<Timestamptz>,
        weight -> Nullable<Float4>,
        link -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        hash -> Varchar,
        name -> Varchar,
        timescale -> Varchar,
        open -> Time,
        close -> Time,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(tasks -> users (assign));

allow_tables_to_appear_in_same_query!(
    arrows,
    invitations,
    permissions,
    tasks,
    users,
);
