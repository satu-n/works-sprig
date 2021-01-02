use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;

#[derive(Deserialize)]
pub struct ReqBody {
    tasks: Vec<i32>,
}

#[derive(Serialize)]
pub struct ResBody {
    text: String,
    msg: String,
}

pub async fn clone(
    req: web::Json<ReqBody>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {
    Ok(HttpResponse::Ok().finish())
}
