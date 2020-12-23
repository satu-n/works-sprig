use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::models;
use crate::errors;
use crate::utils;

#[derive(Serialize)]
pub struct ResHome {
    tasks: Vec<Task>,
}

pub async fn home(
    archives: web::Query<bool>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
