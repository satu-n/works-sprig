use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
use crate::utils;

pub async fn star(
    tid: web::Path<i32>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
