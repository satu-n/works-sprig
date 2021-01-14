use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;

#[derive(Deserialize)]
pub struct ReqBody {
    tasks: Vec<i32>,
    revert: bool,
}

#[derive(Serialize)]
pub struct ResBody {
    tasks: Vec<models::ResTask>,
    info: TasksInfo,
}

pub async fn exec(
    req: web::Json<ReqExec>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
struct TasksInfo {
    count: i32,
    chain: i32,
    revert: bool,
}
