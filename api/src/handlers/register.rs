use actix_web::{web, HttpResponse};
use chrono::NaiveTime;
use diesel::prelude::*;
use serde::Deserialize;

use crate::errors;
use crate::models;
use crate::schema::users;
use crate::utils;

#[derive(Deserialize)]
pub struct ReqBody {
    key: uuid::Uuid,
    email: String,
    password: String,
    reset_pw: bool
}

pub async fn register(
    req: web::Json<ReqBody>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let _ = web::block(move || {
        use crate::schema::permissions::dsl::permissions;
        use crate::schema::users::dsl::{users, email};

        let conn = pool.get().unwrap();
        let req = req.into_inner();
        if req.reset_pw {
            let old_user = users.filter(email.eq(&req.email)).first::<models::User>(&conn)?;
            let alt_user = req.to_alt(&conn)?;
            diesel::update(&old_user).set(&alt_user).execute(&conn)?
        } else {
            let new_user = req.to_new(&conn)?;
            let id = diesel::insert_into(users).values(&new_user).get_result::<models::User>(&conn)?.id;
            let permission = models::Permission {
                subject: id,
                object: id,
                edit: true,
            };
            diesel::insert_into(permissions).values(&permission).execute(&conn)?
        };
        Ok(())
    }).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
    email: String,
    hash: String,
    name: String,
    open: NaiveTime,
    close: NaiveTime,
}

#[derive(AsChangeset)]
#[table_name = "users"]
struct AltUser {
    hash: Option<String>,
}

impl ReqBody {
    fn to_new(&self, conn: &models::Conn) -> Result<NewUser, errors::ServiceError> {
        self.accept(conn)?;
        Ok(NewUser {
            email: self.email.to_owned(),
            hash: utils::hash(&self.password)?,
            name: self.email.to_owned(),
            open: NaiveTime::from_hms(9, 0, 0),
            close: NaiveTime::from_hms(15, 0, 0),
        })
    }
    fn to_alt(&self, conn: &models::Conn) -> Result<AltUser, errors::ServiceError> {
        self.accept(conn)?;
        Ok(AltUser {
            hash: Some(utils::hash(&self.password)?),
        })
    }
    fn accept(&self, conn: &models::Conn) -> Result<(), errors::ServiceError> {
        use crate::schema::invitations::dsl::{invitations, email};

        if let Ok(invitation) = invitations
            .find(&self.key)
            .filter(email.eq(&self.email))
            .first::<models::Invitation>(conn) {
                if chrono::Utc::now() < invitation.expires_at {
                    return Ok(())
                }
                return Err(errors::ServiceError::BadRequest("invitation expired.".into()))
            }
        Err(errors::ServiceError::BadRequest("invitation invalid.".into()))
    }
}
