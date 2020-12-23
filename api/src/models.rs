use diesel::{r2d2::ConnectionManager, PgConnection};
use chrono::{DateTime, NaiveTime, Utc};

use super::schema::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Queryable, Identifiable, Insertable, Debug)]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub forgot_pw: bool,
}

#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub hash: String,
    pub timescale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Identifiable)]
pub struct Duration {
    pub id: i32,
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub owner: i32,
}

#[derive(Queryable, Identifiable)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub assign: i32,
    pub is_done: bool,
    pub is_starred: bool,
    pub weight: Option<f32>,
    pub startable: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Identifiable)]
#[primary_key(source, target)]
pub struct Arrow {
    pub source: i32,
    pub target: i32,
}
