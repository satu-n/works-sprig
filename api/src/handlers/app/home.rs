use actix_web::{error::BlockingError, web, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use std::cmp::min;
use std::collections::HashMap;

use crate::errors;
use crate::models::{self, Selectable};

#[derive(Serialize)]
struct ResHome {
    tasks: Vec<models::ResTask>,
}

#[derive(Deserialize)]
pub struct Q {
    archives: bool,
}

pub async fn home(
    q: web::Query<Q>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res = web::block(move || {
        use crate::schema::arrows::dsl::{arrows, source, target};
        use crate::schema::stripes::dsl::{stripes, owner};
        use crate::schema::tasks::dsl::{tasks, assign, is_done, is_starred, updated_at};
        use crate::schema::users::dsl::{users};
        
        let conn = pool.get().unwrap();
        let is_archives = q.into_inner().archives;
        let _intermediate = tasks
            .filter(assign.eq(&user.id))
            .filter(is_done.eq(&is_archives))
            .inner_join(users)
            .select(models::ResTask::columns())
            .order(updated_at.desc());
        let res_tasks = if is_archives {
            _intermediate
            .order(is_starred.desc())
            .limit(100)
            .load::<models::ResTask>(&conn)?
        } else {
            let _res_tasks = _intermediate.load::<models::ResTask>(&conn)?;
            let ids = _res_tasks.iter().map(|t| t.id).collect::<Vec<i32>>();
            let _arrows = arrows
                .filter(source.eq_any(&ids))
                .filter(target.eq_any(&ids))
                .load::<models::Arrow>(&conn)?;
            let _stripes = stripes
                .filter(owner.eq(&user.id))
                .load::<models::Stripe>(&conn)?;
            let mut sorter = Sorter::new(_res_tasks, _arrows, _stripes);
            sorter.exec();
            sorter.tasks
        };
        Ok(ResHome { tasks: res_tasks })
    }).await;

    match res {
        Ok(res_home) => Ok(HttpResponse::Ok().json(res_home)),
        Err(err) => match err {
            BlockingError::Error(service_error) => {
                dbg!(&service_error);
                Err(service_error)
            },
            BlockingError::Canceled => Err(errors::ServiceError::InternalServerError),
        },
    }
}

struct Sorter {
    now: DateTime<Utc>,
    tasks: Vec<models::ResTask>,
    arrows: Vec<models::Arrow>,
    stripes: Vec<models::Stripe>,
}

impl Sorter {
    fn new(
        tasks: Vec<models::ResTask>,
        arrows: Vec<models::Arrow>,
        stripes: Vec<models::Stripe>,
    ) -> Self { Self {
            now: Utc::now(),
            tasks: tasks,
            arrows: arrows,
            stripes: stripes,
        }
    }
    fn exec(&mut self) {
        let mut sub = self.to_sub();
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
    fn to_sub(&self) -> SubSorter {
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
            entries: map.keys().cloned().collect::<Vec<i32>>(),
            arrows: self.arrows.clone().into(),
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

struct Tid {
    id: i32,
}

struct Path {
    path: Vec<i32>,
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
        self.entries.iter().cloned()
        .filter(|id| Tid::from(*id).is_leaf(&self.arrows))
        .filter(|id| {
            self.map[&id].startable.map(|t| t < self.cursor).unwrap_or(true)
        }).collect::<Vec<i32>>()
    }
    fn priority(&self, id: &i32) -> Option<i64> {
        self.paths(&id).iter_mut()
        .map(|path| self.priority_by(path))
        .max().unwrap_or_default()
    }
    fn priority_by(&self, path: &mut Path) -> Option<i64> {
        let mut cursor = i64::MAX;
        while let Some(id) = path.path.pop() {
            cursor = if let Some(deadline) = self.map[&id].deadline {
                min(cursor, deadline)
            } else {
                cursor
            };
            cursor -= self.map[&id].weight.unwrap_or_default();
        }
        if cursor == i64::MAX {
            return None
        }
        Some(self.cursor - cursor)
    }
    fn paths(&self, id: &i32) -> Vec<Path> {
        let mut paths = Tid::from(*id).paths(&self.arrows);
        for path in paths.iter_mut() {
            while let Some(last) = path.path.pop() {
                if self.map[&last].deadline.is_some() {
                    path.path.push(last);
                    break
                }
            }
        }
        paths.retain(|path| !path.path.is_empty());
        paths
    }
}

impl From<i32> for Tid {
    fn from(id: i32) -> Self {
        Self { id: id }
    }
}

impl From<Vec<i32>> for Path {
    fn from(path: Vec<i32>) -> Self {
        Self { path: path }
    }
}

impl Tid {
    fn is_leaf(&self, arrows: &models::Arrows) -> bool {
        arrows.arrows.iter().all(|arw| arw.target != self.id)
    }
    fn is_root(&self, arrows: &models::Arrows) -> bool {
        arrows.arrows.iter().all(|arw| arw.source != self.id)
    }
    fn paths(&self, arrows: &models::Arrows) -> Vec<Path> {
        let map = arrows.to_map();
        let mut results: Vec<Path> = Vec::new();
        let mut remains: Vec<i32> = Vec::new();
        let mut re_map: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut cursor = self.id;
        let mut path: Vec<i32> = Vec::new();
        'main: loop {
            path.push(cursor);
            let mut successors = map[&cursor].clone();
            if let Some(suc) = successors.pop() {
                remains.push(cursor);
                re_map.insert(cursor, successors);
                cursor = suc;
                continue
            }
            results.push(Path::from(path.clone()));
            while let Some(rem) = remains.pop() {
                while cursor != rem {
                    cursor = path.pop().unwrap();
                }
                path.push(cursor);
                if let Some(suc) = re_map.get_mut(&cursor).unwrap().pop() {
                    remains.push(cursor);
                    cursor = suc;
                    continue 'main
                }
            }
            break
        }
        results
    }
}
