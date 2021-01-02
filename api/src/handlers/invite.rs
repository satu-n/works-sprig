use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;

use crate::errors;
use crate::models;

#[derive(Deserialize)]
pub struct ReqBody {
    email: String,
    forgot_pw: bool,
}

pub async fn invite(
    req: web::Json<ReqBody>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let _ = web::block(move || {
        let conn = pool.get().unwrap();
        let invitation: models::Invitation = req.into_inner().accept(&conn)?;
        dbg!(&invitation);
        super::email::send(&invitation)
    }).await?;

    Ok(HttpResponse::Ok().finish())
}

impl ReqBody {
    fn accept(self,
        conn: &models::Conn,
    ) -> Result<models::Invitation, errors::ServiceError> {
        use diesel::dsl::{select, exists};
        use crate::schema::invitations::dsl::invitations;
        use crate::schema::users::dsl::{users, email};

        let user_exists: bool = select(exists(users.filter(email.eq(&self.email)))).get_result(conn)?;
        if user_exists && !self.forgot_pw {
            return Err(errors::ServiceError::BadRequest("user already exists".into()))
        }
        if !user_exists && self.forgot_pw {
            return Err(errors::ServiceError::BadRequest("user does not exist yet".into()))
        }
        let invitation: models::Invitation = self.into();

        Ok(diesel::insert_into(invitations).values(&invitation).get_result(conn)?)
    }
}

impl From<ReqBody> for models::Invitation {
    fn from(req: ReqBody) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            email: req.email,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            forgot_pw: req.forgot_pw,
        }
    }
}
