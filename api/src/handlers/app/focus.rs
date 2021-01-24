use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Serialize;

use crate::errors;
use crate::models::{self, Selectable};

#[derive(Serialize)]
pub struct ResBody {
    pred: Vec<models::ResTask>,
    succ: Vec<models::ResTask>,
}

pub async fn focus(
    tid: web::Path<i32>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res_body = web::block(move || {
        use diesel::dsl::exists;
        use crate::schema::arrows::dsl::*;
        use crate::schema::permissions::dsl::*;
        use crate::schema::tasks::dsl::{tasks, id, assign};
        use crate::schema::users::dsl::users;

        let conn = pool.get().unwrap();
        let tid = tid.into_inner();
        let query = tasks
        .filter(exists(permissions
            .filter(subject.eq(&user.id))
            .filter(object.eq(assign))
        ))
        .inner_join(users)
        .select(models::SelTask::columns());

        let pred = query
        .filter(exists(arrows.filter(source.eq(id)).filter(target.eq(&tid))))
        .load::<models::SelTask>(&conn)?
        .into_iter().map(|t| t.to_res()).collect();

        let succ = query
        .filter(exists(arrows.filter(source.eq(&tid)).filter(target.eq(id))))
        .load::<models::SelTask>(&conn)?
        .into_iter().map(|t| t.to_res()).collect();

        Ok(ResBody {
            pred: pred,
            succ: succ,
        })
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}
