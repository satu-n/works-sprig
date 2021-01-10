use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use std::cmp::min;
use std::collections::HashMap;

use crate::errors;
use crate::models::{self, Selectable};

#[derive(Deserialize)]
pub struct Q {
    pub option: Option<String>,
}

#[derive(Serialize)]
struct ResBody {
    tasks: Vec<models::ResTask>,
    config: Config,
}

pub async fn home( // FIXME 500 ISE
    q: web::Query<Q>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res_body = web::block(move || {
        let conn = pool.get().unwrap();
        let config = q.into_inner().config();
        let res_tasks = config.query(&user, &conn)?;

        Ok(ResBody {
            tasks: res_tasks,
            config: config,
        })
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}

#[derive(Serialize)]
pub enum Config {
    Archives,
    Roots,
    Leaves,
    Home,
}

impl Q {
    fn config(&self) -> Config {
        match self.option.as_deref() {
            Some("archives") => Config::Archives,
            Some("roots")    => Config::Roots,
            Some("leaves")   => Config::Leaves,
            _                => Config::Home,
        }
    }
}

impl Config {
    pub fn query(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<Vec<models::ResTask>, errors::ServiceError> {
        use crate::schema::stripes::dsl::{stripes, owner};
        use crate::schema::tasks::dsl::{tasks, assign, is_archived, is_starred, updated_at};
        use crate::schema::users::dsl::users;

        let archives = if let Self::Archives = self { true } else { false };

        let _intermediate = tasks
            .filter(assign.eq(&user.id))
            .filter(is_archived.eq(&archives))
            .inner_join(users)
            .select(models::ResTask::columns());
        let res_tasks = if archives {
            _intermediate
            .order((is_starred.desc(), updated_at.desc()))
            .limit(100)
            .load::<models::ResTask>(conn)?
        } else {
            let _res_tasks = _intermediate
                .order(updated_at.desc())
                .load::<models::ResTask>(conn)?;

            let arrows = models::Arrows::among(&_res_tasks, conn)?;
            let mut sorter = Sorter::new(_res_tasks,
                stripes
                .filter(owner.eq(&user.id))
                .load::<models::Stripe>(conn)?
            );
            sorter.exec(arrows.clone());
            self.filter(&mut sorter.tasks, &arrows);
            sorter.tasks
        };
        Ok(res_tasks)
    }
    fn filter(&self,
        tasks: &mut Vec<models::ResTask>,
        arrows: &models::Arrows,
    ) {
        match self {
            Self::Leaves => {
                tasks.retain(|t| models::Tid::from(t.id).is(models::LR::Leaf, arrows))
            },
            Self::Roots => {
                tasks.retain(|t| models::Tid::from(t.id).is(models::LR::Root, arrows))
            },
            _ => (),
        }
    }
}

struct Sorter {
    now: DateTime<Utc>,
    tasks: Vec<models::ResTask>,
    stripes: Vec<models::Stripe>,
}

impl Sorter {
    fn new(
        tasks: Vec<models::ResTask>,
        stripes: Vec<models::Stripe>,
    ) -> Self { Self {
            now: Utc::now(),
            tasks: tasks,
            stripes: stripes,
        }
    }
    fn exec(&mut self, arrows: models::Arrows) {
        let mut sub = self.to_sub(arrows);
        sub.exec();
        // set priority
        for t in &mut self.tasks {
            if let Some(p) = sub.map[&t.id].priority {
                t.priority = Some(p as f32 / 3600.0) // hours from seconds
            }
        }
        self.tasks.sort_by(|a, b| sub.map[&b.id].rank.cmp(&sub.map[&a.id].rank));
        self.tasks.sort_by(|a, b| b.is_starred.cmp(&a.is_starred));
    }
    fn to_sub(&self, arrows: models::Arrows) -> SubSorter {
        let mut map = HashMap::new();
        for t in &self.tasks {
            map.insert(t.id, SubTask {
                startable: t.startable.map(|dt| self.splice(dt)),
                deadline: t.deadline.map(|dt| self.splice(dt)),
                priority: None,
                weight: t.weight.map(|w| (w * 3600.0) as i64),
                rank: None,
            });
        }
        SubSorter {
            cursor: 0,
            entries: map.keys().copied().collect::<Vec<i32>>(),
            arrows: arrows,
            map: map,
        }
    }
    // TODO splice
    fn splice(&self, dt: DateTime<Utc>) -> i64 {
        let days = dt.signed_duration_since(self.now).num_days();
        let daily = 6 * 3600;
        let alpha = 0;
        daily * days + alpha
    }
}

struct SubSorter {
    cursor: i64,
    entries: Vec<i32>,
    arrows: models::Arrows,
    map: HashMap<i32, SubTask>,
}

struct SubTask {
    startable: Option<i64>,
    deadline: Option<i64>,
    priority: Option<i64>,
    weight: Option<i64>,
    rank: Option<i32>,
}

struct Winner {
    id: i32,
    priority: i64,
}

impl SubSorter {
    fn exec(&mut self) {
        let mut rank = 0;
        while let Some(win) = self.winner() {
            self.map.get_mut(&win.id).unwrap().priority = Some(win.priority);
            self.map.get_mut(&win.id).unwrap().rank = Some(rank);
            rank -= 1;
            self.cursor += self.map[&win.id].weight.unwrap_or_default();
            self.entries.retain(|id| *id != win.id);
            self.arrows.arrows.retain(|arw| arw.source != win.id);
        }
    }
    fn winner(&self) -> Option<Winner> {
        let mut winner = None::<Winner>;
        for id in self.startables() {
            if let Some(priority) = self.priority(&id) {
                if let Some(win) = &winner {
                    if priority <= win.priority { continue }
                }
                winner = Some(Winner {
                    id: id,
                    priority: priority,
                })
            }
        }
        winner
    }
    fn startables(&self) -> Vec<i32> {
        self.entries.iter().copied()
        .filter(|id| models::Tid::from(*id).is(models::LR::Leaf, &self.arrows))
        .filter(|id| self.map[&id].startable.map(|t| t < self.cursor).unwrap_or(true))
        .collect::<Vec<i32>>()
    }
    fn priority(&self, id: &i32) -> Option<i64> {
        self.paths(&id).iter()
        .map(|path| self.priority_by(path))
        .max().unwrap_or_default()
    }
    fn priority_by(&self, path: &models::Path) -> Option<i64> {
        let mut cursor = i64::MAX;
        for id in path.iter().rev() {
            if let Some(deadline) = self.map[&id].deadline {
                cursor = min(cursor, deadline)
            }
            cursor -= self.map[&id].weight.unwrap_or_default()
        }
        if cursor == i64::MAX {
            return None
        }
        Some(self.cursor - cursor)
    }
    fn paths(&self, id: &i32) -> Vec<models::Path> {
        let mut paths = models::Tid::from(*id).paths_to(models::LR::Root, &self.arrows);
        for path in &mut paths {
            while let Some(last) = path.pop() {
                if self.map[&last].deadline.is_some() {
                    path.push(last);
                    break
                }
            }
        }
        paths.retain(|path| !path.is_empty());
        paths
    }
}
