use actix_identity::Identity;
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqAuth {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct ResAuth {
    email: String,
}

impl models::AuthedUser {
    fn to_res(&self, conn: &models::Conn) -> Result<ResAuth, errors::ServiceError> {
        use crate::schema::users::dsl::users;

        let user = users.find(self.id).first::<models::User>(conn)?;
        Ok(ResAuth { email: user.email })
    }
}

impl ReqAuth {
    fn to_authed(&self, conn: &models::Conn) -> Result<models::AuthedUser, errors::ServiceError> {
        use crate::schema::users::dsl::{users, email};

        let user = users
            .filter(email.eq(&self.email))
            .first::<models::User>(conn)?;
        if utils::verify(&user.hash, &self.password)? {
            return Ok(models::AuthedUser { id: user.id })
        }
        Err(errors::ServiceError::Unauthorized)
    }
}

pub async fn get_me(
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        let conn = pool.get().unwrap();
        user.to_res(&conn)
    }).await;

    match res {
        Ok(res_auth) => {
            Ok(HttpResponse::Ok().json(&res_auth))
        },
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
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
            let identity = authed_user.id.to_string();
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
