use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
use crate::utils;

#[derive(Serialize)]
pub struct ResFocus {
    tasks: Vec<Task>,
}

pub async fn focus(
    tid: web::Path<i32>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
