use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqClone {
    tasks: Vec<i32>,
}

#[derive(Serialize)]
pub struct ResClone {
    text: String,
}

pub async fn clone(
    req: web::Json<ReqClone>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
