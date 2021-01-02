use actix_web::{web, HttpResponse};
use diesel::prelude::*;

use crate::errors;
use crate::models;

pub async fn star(
    tid: web::Path<i32>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
