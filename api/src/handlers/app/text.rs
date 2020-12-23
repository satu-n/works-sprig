use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::models;
use crate::errors;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqText {
    text: String,
}

#[derive(Serialize)]
pub struct ResText {}

pub async fn text(
    req: web::Json<ReqText>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
