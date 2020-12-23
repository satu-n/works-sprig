use actix_web::{error::BlockingError, web, HttpResponse};
use actix_identity::Identity;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::models;
use crate::errors;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqAuth {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct AuthedUser {
    email: String,
}

impl ReqAuth {
    fn to_authed(&self, conn: &models::Conn) -> Result<AuthedUser, errors::ServiceError> {
        use crate::schema::users::dsl::{users, email};

        let user = users
            .filter(email.eq(&self.email))
            .first::<models::User>(conn)?;
        if utils::verify(&user.hash, &self.password)? {
            return Ok(AuthedUser { email: user.email })
        }
        Err(errors::ServiceError::Unauthorized)
    }
}

pub async fn get_me(id: Identity) -> Result<HttpResponse, errors::ServiceError> {
    if let Some(identity) = id.identity() {
        if let Ok(authed_user) = serde_json::from_str::<AuthedUser>(&identity) {
            return Ok(HttpResponse::Ok().json(&authed_user))
        }
    }
    Err(errors::ServiceError::Unauthorized)
}

pub async fn login(
    req_auth: web::Json<ReqAuth>,
    id: Identity,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        let conn = pool.get().unwrap();
        req_auth.into_inner().to_authed(&conn)
    }).await;

    match res {
        Ok(authed_user) => {
            let identity = serde_json::to_string(&authed_user).unwrap();
            id.remember(identity);
            Ok(HttpResponse::Ok().finish())
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
}

pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().finish()
}
