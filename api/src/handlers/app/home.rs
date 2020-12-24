use actix_web::{error::BlockingError, web, HttpResponse};
// use chrono::{DateTime, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
// use crate::utils;

#[derive(Serialize)]
struct ResHome {
    tasks: Vec<models::ResTask>,
}

#[derive(Deserialize)]
pub struct Q {
    archives: bool,
}

pub async fn home(
    q: web::Query<Q>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        use crate::schema::tasks::dsl::{tasks, assign, is_done, is_starred, updated_at};

        let conn = pool.get().unwrap();
        let is_archives = q.into_inner().archives;
        let _intermediate = tasks
            .filter(assign.eq(&user.id))
            .filter(is_done.eq(&is_archives));
        let _tasks = if is_archives {
            _intermediate
            .order((is_starred.desc(), updated_at.desc()))
            .limit(100)
            .load::<models::Task>(&conn)?
        } else {
            _intermediate
            // .order((is_starred.desc(), priority.desc()))
            .load::<models::Task>(&conn)?
        };
        let mut res = Vec::new();
        for t in _tasks {
            res.push(t.to_res(&user, &conn)?)
        }
        Ok(ResHome { tasks: res })
    }).await;

    match res {
        Ok(res_home) => Ok(HttpResponse::Ok().json(res_home)),
        Err(err) => match err {
            BlockingError::Error(service_error) => {
                dbg!(&service_error);
                Err(service_error)
            },
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
}
