use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::models;
use crate::errors;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqExec {
    tasks: Vec<i32>,
    revert: bool,
}

#[derive(Serialize)]
pub struct ResExec {}

pub async fn exec(
    req: web::Json<ReqExec>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
