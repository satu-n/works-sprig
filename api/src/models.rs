use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use chrono::{DateTime, NaiveTime, Utc};
// use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection}; // TODO redundant PgConnection ?
use diesel::sql_types::{Nullable, Float};
use diesel::expression::{bound, IntoSql};
use futures::future::{err, ok, Ready};
use serde::{Serialize};
use std::collections::HashMap;

use crate::errors;
use crate::schema::*;
// use crate::utils;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// FROM SCHEMA

#[derive(Queryable, Identifiable)]
#[primary_key(source, target)]
pub struct Arrow {
    pub source: i32,
    pub target: i32,
}

#[derive(Queryable, Identifiable, Insertable, Debug)]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub forgot_pw: bool,
}

#[derive(Queryable, Identifiable)]
pub struct Stripe {
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
    pub startable: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub weight: Option<f32>,
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

#[derive(Queryable, Serialize)]
pub struct ResTask {
    pub id: i32,
    pub title: String,
    pub assign: String,
    pub is_done: bool,
    pub is_starred: bool,
    pub startable: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub priority: Option<f32>,
    pub weight: Option<f32>,
    pub link: Option<String>,
}

pub trait Selectable {
    type Columns;
    fn columns() -> Self::Columns;
}

impl Selectable for ResTask {
    type Columns = (
        tasks::id,
        tasks::title,
        users::email,
        tasks::is_done,
        tasks::is_starred,
        tasks::startable,
        tasks::deadline,
        bound::Bound<Nullable<Float>, Option<f32>>,
        tasks::weight,
        tasks::link,
    );
    fn columns() -> Self::Columns {(
        tasks::id,
        tasks::title,
        users::email,
        tasks::is_done,
        tasks::is_starred,
        tasks::startable,
        tasks::deadline,
        None::<f32>.into_sql::<Nullable<Float>>(),
        tasks::weight,
        tasks::link,
    )}
}

pub struct Arrows {
    pub arrows: Vec<Arrow>,
}

impl From<Vec<Arrow>> for Arrows {
    fn from(arrows: Vec<Arrow>) -> Self {
        Self { arrows: arrows }
    }
}

impl Arrows {
    pub fn to_map(&self) -> HashMap<i32, Vec<i32>> {
        let mut map: HashMap<i32, Vec<i32>> = HashMap::new();
        for arw in self.arrows.iter() {
            let targets = map.entry(arw.source).or_default();
            targets.push(arw.target);
        }
        map
    }
}
