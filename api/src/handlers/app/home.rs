use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::models;
use crate::errors;
use crate::utils;

// #[derive(Serialize)]
// pub struct ResHome {
//     tasks: Vec<Task>,
// }

#[derive(Deserialize)]
pub struct Q {
    archives: bool,
}

pub async fn home(
    q: web::Query<Q>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().json(q.into_inner().archives))
}
