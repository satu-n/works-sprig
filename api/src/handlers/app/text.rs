use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
use crate::utils;
use super::home,
use super::parser,

#[derive(Deserialize)]
pub struct ReqText {
    text: String,
}

#[derive(Serialize)]
pub struct ResText {
    tasks: Vec<models::ResTask>,
    msg: String, // for Ok only. for Err, use HttpResponse::BadRequest
}

pub async fn text(
    req: web::Json<ReqText>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let text = req.into_inner().text.parse::<Text>?;

    let res = web::block(move || {
        let mut msg = String::new();
        let res_tasks = match text {
            Command::Select(condition) => condition.extract(&user, &conn)?,
            ReqTasks(tasks) => {
                msg = tasks.accept(&user, &conn)?.upsert(&user, &conn)?;
                home::Q { archives: false }.query(&user, &conn)?
            }
        }
        Ok(ResText {
            tasks: res_tasks,
            msg: msg,
        })
    }).await;

    match res {
        Ok(res_text) => Ok(HttpResponse::Ok().json(res_text)),
        Err(err) => match err {
            BlockingError::Error(service_error) => {
                dbg!(&service_error);
                Err(service_error)
            },
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
}

enum Text {
    Command(Command),
    ReqTasks(ReqTasks),
}

enum Command {
    Select(Condition),
}

struct Condition {

}

struct ReqTasks {
    tasks: Vec<ReqTask>,
}

struct ReqTask {

}

impl Condition {
    fn extract() -> Result<Vec<models::ResTask>, errors::ServiceError> {}
}

impl ReqTasks {
    fn accept() -> Result<HappySet, errors::ServiceError> {}
}

struct HappySet {
    news: Vec<NewTask>,
    alts: Hashmap<i32, AltTask>,
}

#[derive(Insertable)]
struct NewTask {

}

#[derive(AsChangeset)]
struct AltTask {

}

impl HappySet {
    fn upsert() -> Result<String, errors::ServiceError> {}
}
