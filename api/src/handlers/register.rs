use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;

use crate::errors;
use crate::models;
use crate::schema::users;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqUser {
    key: uuid::Uuid,
    email: String,
    password: String,
    reset_pw: bool
}

#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    email: String,
    hash: String,
}

#[derive(AsChangeset)]
#[table_name = "users"]
struct AltUser {
    hash: Option<String>,
}

impl ReqUser {
    fn to_new(&self, conn: &models::Conn) -> Result<NewUser, errors::ServiceError> {
        self.validate(conn)?;
        Ok(NewUser {
            email: self.email.to_owned(),
            hash: utils::hash(&self.password)?,
        })
    }
    fn to_alt(&self, conn: &models::Conn) -> Result<AltUser, errors::ServiceError> {
        self.validate(conn)?;
        Ok(AltUser {
            hash: Some(utils::hash(&self.password)?),
        })
    }
    fn validate(&self, conn: &models::Conn) -> Result<(), errors::ServiceError> {
        use crate::schema::invitations::dsl::{invitations, email};

        if let Ok(invitation) = invitations
            .find(&self.key)
            .filter(email.eq(&self.email))
            .first::<models::Invitation>(conn) {
                if chrono::Utc::now() < invitation.expires_at {
                    return Ok(())
                }
                return Err(errors::ServiceError::BadRequest("invitation expired".into()))
            }
        Err(errors::ServiceError::BadRequest("invitation invalid".into()))
    }
}

pub async fn register(
    req_user: web::Json<ReqUser>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        use crate::schema::users::dsl::{users, email};

        let conn = pool.get().unwrap();
        let req = req_user.into_inner();
        if req.reset_pw {
            let old_user = users.filter(email.eq(&req.email)).first::<models::User>(&conn)?;
            let alt_user = req.to_alt(&conn)?;
            diesel::update(&old_user).set(&alt_user).execute(&conn)?
        } else {
            let new_user = req.to_new(&conn)?;
            diesel::insert_into(users).values(&new_user).execute(&conn)?
        };
        Ok(())
    }).await;

    match res {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(service_error) => {
                dbg!(&service_error);
                Err(service_error)
            },
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
}
