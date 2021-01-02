use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Serialize;

use crate::errors;
use crate::models;

#[derive(Serialize)]
pub struct ResBody {
    tasks: Vec<models::ResTask>,
}

pub async fn focus(
    tid: web::Path<i32>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
