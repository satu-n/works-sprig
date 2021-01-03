use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection}; // TODO redundant PgConnection ?
use diesel::sql_types::{Nullable, Float};
use diesel::expression::{bound, IntoSql};
use futures::future::{err, ok, Ready};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::ops::Not;

use crate::errors;
use crate::schema::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Conn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

// FROM SCHEMA

#[derive(Queryable, Identifiable, Insertable, Clone)]
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

#[derive(Queryable, Identifiable, Insertable)]
#[primary_key(subject, object)]
pub struct Permission {
    pub subject: i32,
    pub object: i32,
    pub edit: bool,
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
    pub name: String,
    pub timescale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// VARIATIONS

#[derive(Identifiable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct AuthedUser {
    pub id: i32,
    pub tz: Tz,
}

impl FromRequest for AuthedUser {
    type Config = ();
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        use actix_identity::RequestIdentity;
        if let Some(identity) = req.get_identity() {
            if let Ok(user) = serde_json::from_str::<Self>(&identity) {
                return ok(user)
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
        users::name,
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
        users::name,
        tasks::is_done,
        tasks::is_starred,
        tasks::startable,
        tasks::deadline,
        None::<f32>.into_sql::<Nullable<Float>>(),
        tasks::weight,
        tasks::link,
    )}
}

#[derive(Clone)]
pub struct Arrows {
    pub arrows: Vec<Arrow>,
}

impl From<Vec<Arrow>> for Arrows {
    fn from(arrows: Vec<Arrow>) -> Self {
        Self { arrows: arrows }
    }
}

impl Arrows {
    pub fn map_to(&self, lr: LR) -> HashMap<i32, Vec<i32>> {
        let mut map: HashMap<i32, Vec<i32>> = HashMap::new();
        for arw in self.arrows.iter() {
            map.entry(arw.trace_to(!lr)).or_default().push(arw.trace_to(lr));
        }
        map
    }
}

#[derive(Clone, Copy)]
pub enum LR {
    Leaf,
    Root,
}

impl Not for LR {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Leaf => Self::Root,
            Self::Root => Self::Leaf,
        }
    }
}

impl Arrow {
    pub fn trace_to(&self, lr: LR) -> i32 {
        match lr {
            LR::Leaf => self.source,
            LR::Root => self.target,
        }
    }
}

#[derive(Identifiable)]
#[table_name = "tasks"]
pub struct Tid {
    pub id: i32,
}

pub type Path = Vec<i32>;

impl From<i32> for Tid {
    fn from(id: i32) -> Self {
        Self { id: id }
    }
}

impl Tid {
    pub fn is(&self, lr: LR, arrows: &Arrows) -> bool {
        arrows.arrows.iter().all(|arw| arw.trace_to(!lr) != self.id)
    }
    pub fn paths_to(&self, lr: LR, arrows: &Arrows) -> Vec<Path> {
        let map = arrows.map_to(lr);
        let mut results: Vec<Path> = Vec::new();
        let mut remains: Vec<i32> = Vec::new();
        let mut re_map: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut cursor = self.id;
        let mut path: Vec<i32> = Vec::new();
        'main: loop {
            if path.contains(&cursor) { // got cycle instead of path
                results.clear();
                break
            }
            path.push(cursor);
            let mut destinations = map[&cursor].clone();
            if let Some(dest) = destinations.pop() {
                remains.push(cursor);
                re_map.insert(cursor, destinations);
                cursor = dest;
                continue
            }
            results.push(Path::from(path.clone()));
            while let Some(rem) = remains.pop() {
                while cursor != rem {
                    cursor = path.pop().unwrap();
                }
                path.push(cursor);
                if let Some(dest) = re_map.get_mut(&cursor).unwrap().pop() {
                    remains.push(cursor);
                    cursor = dest;
                    continue 'main
                }
            }
            break
        }
        results
    }
    pub fn nodes_to(&self, lr: LR, arrows: &Arrows) -> Vec<i32> {
        let mut nodes = self.paths_to(lr, arrows).into_iter().flatten().collect::<Vec<i32>>();
        nodes.sort();
        nodes.dedup();
        nodes
    }
}

impl Arrows {
    pub fn among(
        tasks: &Vec<ResTask>,
        conn: &Conn,
    ) -> Result<Self, errors::ServiceError> {
        use crate::schema::arrows::dsl::*;

        let ids = tasks.iter().map(|t| t.id).collect::<Vec<i32>>();
        Ok(arrows
            .filter(source.eq_any(&ids))
            .filter(target.eq_any(&ids))
            .load::<Arrow>(conn)?
            .into()
        )
    }
    pub fn paths(&self) -> Vec<Path> {
        self.list(LR::Leaf).iter().flat_map(|leaf| Tid::from(*leaf).paths_to(LR::Root, &self)).collect()
    }
    pub fn list(&self, lr: LR) -> Vec<i32> {
        self.nodes().into_iter().filter(|id| Tid::from(*id).is(lr, &self)).collect()
    }
    pub fn nodes(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for arw in &self.arrows {
            ids.push(arw.target);
            ids.push(arw.source);
        }
        ids.sort();
        ids.dedup();
        ids
    }
    pub fn has_cycle(&self) -> bool {
        if self.list(LR::Leaf).is_empty() || self.list(LR::Root).is_empty() {
            return true
        }
        self.list(LR::Leaf).iter().any(|leaf| Tid::from(*leaf).paths_to(LR::Root, &self).is_empty())
    }
}

impl AuthedUser {
    pub fn globalize(&self, dt: &NaiveDateTime) -> DateTime<Utc> {
        self.tz.from_local_datetime(dt).unwrap().with_timezone(&Utc)
    }
    pub fn localize(&self, dt: &DateTime<Utc>) -> NaiveDateTime {
        dt.with_timezone(&self.tz).naive_local()
    }
}
