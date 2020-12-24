use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;

use crate::errors;
use crate::models;

#[derive(Deserialize)]
pub struct ReqInvitation {
    email: String,
    forgot_pw: bool,
}

impl From<ReqInvitation> for models::Invitation {
    fn from(req: ReqInvitation) -> Self {
        models::Invitation {
            id: uuid::Uuid::new_v4(),
            email: req.email,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            forgot_pw: req.forgot_pw,
        }
    }
}

pub async fn invite(
    req_invitation: web::Json<ReqInvitation>,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        use diesel::dsl::*;
        use crate::schema::users::dsl::{users, email};
        use crate::schema::invitations::dsl::invitations;

        let conn = pool.get().unwrap();
        let req = req_invitation.into_inner();
        let user_exists: bool = select(exists(users.filter(email.eq(&req.email)))).get_result(&conn)?;
        if req.forgot_pw != user_exists {
            if user_exists {
                return Err(errors::ServiceError::BadRequest("user already exists".into()))
            } else {
                return Err(errors::ServiceError::BadRequest("user does not exist yet".into()))
            }
        }
        let new_invitation = models::Invitation::from(req);
        let invitation = diesel::insert_into(invitations).values(&new_invitation).get_result(&conn)?;
        dbg!(&invitation);

        super::email::send(&invitation)
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
