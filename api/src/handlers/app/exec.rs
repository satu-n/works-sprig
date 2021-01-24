use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::errors;
use crate::models;

#[derive(Deserialize)]
pub struct ReqBody {
    tasks: Vec<i32>,
    revert: bool,
}

#[derive(Serialize)]
pub struct ResBody {
    count: usize,
    chain: usize,
}

pub async fn exec(
    req: web::Json<ReqBody>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res_body = web::block(move || {
        use diesel::dsl::exists;
        use crate::schema::arrows::dsl::arrows;
        use crate::schema::permissions::dsl::*;
        use crate::schema::tasks::dsl::{tasks, id, assign, is_archived};

        let conn = pool.get().unwrap();
        let req = req.into_inner();
        let _arrows: models::Arrows = arrows.load::<models::Arrow>(&conn)?.into();
        let entries = req.verify(&user, &conn)?;
        let targets = entries.iter()
        .flat_map(|tid| models::Tid::from(*tid).nodes_to(
            if req.revert { models::LR::Root } else { models::LR::Leaf }
            , &_arrows)
        ).collect::<Vec<i32>>();

        let count = diesel::update(tasks
            .filter(exists(permissions
                .filter(subject.eq(&user.id))
                .filter(object.eq(assign))
                .filter(edit)
            ))
            .filter(is_archived.eq(&req.revert))
            .filter(id.eq_any(&targets))
        ).set(is_archived.eq(&!req.revert)).execute(&conn)?;

        Ok(ResBody {
            count: count,
            chain: count - entries.len(),
        })
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}

impl ReqBody {
    fn verify(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<Vec<i32>, errors::ServiceError> {
        use diesel::dsl::exists;
        use crate::schema::permissions::dsl::*;
        use crate::schema::tasks::dsl::{tasks, id, assign, is_archived};

        if let Some(tid) = tasks
        .filter(id.eq_any(&self.tasks))
        .filter(
            exists(permissions
                .filter(subject.eq(&user.id))
                .filter(object.eq(assign))
                .filter(edit)
            ).eq(false)
        )
        .select(id)
        .first::<i32>(conn).ok() {
            return Err(errors::ServiceError::BadRequest(format!(
                "#{}: no edit permission.",
                tid
            )))
        }
        Ok(tasks
            .filter(is_archived.eq(&self.revert))
            .filter(id.eq_any(&self.tasks))
            .select(id)
            .load::<i32>(conn)?
        )
    }
}
