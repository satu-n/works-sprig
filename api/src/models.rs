use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use chrono::{DateTime, NaiveTime, Utc};
use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection}; // TODO redundant PgConnection ?
use futures::future::{err, ok, Ready};
use serde::{Serialize};

use crate::errors;
use crate::schema::*;
use crate::utils;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// FROM SCHEMA

#[derive(Queryable, Identifiable)]
#[primary_key(source, target)]
pub struct Arrow {
    pub source: i32,
    pub target: i32,
}

#[derive(Queryable, Identifiable)]
pub struct Duration {
    pub id: i32,
    pub open: NaiveTime,
    pub close: NaiveTime,
    pub owner: i32,
}

#[derive(Queryable, Identifiable, Insertable, Debug)]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub forgot_pw: bool,
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
pub struct User {
    pub id: i32,
    pub email: String,
    pub hash: String,
    pub timescale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// VARIATIONS

#[derive(Identifiable)]
#[table_name = "users"]
pub struct AuthedUser {
    pub id: i32,
}

impl FromRequest for AuthedUser {
    type Config = ();
    type Error = Error;
    type Future = Ready<Result<AuthedUser, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        use actix_identity::RequestIdentity;
        if let Some(identity) = req.get_identity() {
            if let Ok(id) = identity.parse::<i32>() {
                return ok(AuthedUser { id: id })
            }
        }
        err(errors::ServiceError::Unauthorized.into())
    }
}

#[derive(Serialize)]
pub struct ResTask {
    id: i32,
    title: String,
    assign: Option<String>,
    is_done: bool,
    is_starred: bool,
    weight: Option<f32>,
    startable: Option<DateTime<Utc>>,
    deadline: Option<DateTime<Utc>>,
    link: Option<String>,
}

impl Task {
    pub fn to_res(
        self,
        user: &AuthedUser,
        conn: &Conn
    ) -> Result<ResTask, errors::ServiceError> {
        let assign = if self.assign == user.id { None } else { Some(
            crate::schema::users::dsl::users
            .find(&self.assign)
            .first::<User>(conn)?
            .mailbox()
        )};
        Ok(ResTask {
            id: self.id,
            title: self.title,
            assign: assign,
            is_done: self.is_done,
            is_starred: self.is_starred,
            weight: self.weight,
            startable: self.startable,
            deadline: self.deadline,
            link: self.link,
        })
    }
}

impl User {
    pub fn mailbox(&self) -> String {
        utils::mailbox(&self.email)
    }
}
