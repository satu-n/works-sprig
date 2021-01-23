use actix_web::{web, HttpResponse};
use chrono::{Date, DateTime, Duration, Utc};
use chrono_tz::Tz;
use diesel::prelude::*;
use gcollections::ops::{Cardinality, Intersection};
use interval::interval_set::ToIntervalSet;
use interval::interval_set::{IntervalSet};
use serde::{Serialize, Deserialize};
use std::cmp::min;
use std::collections::HashMap;

use crate::errors;
use crate::models::{self, Selectable};

#[derive(Deserialize, Serialize)]
pub struct Q {
    pub option: Option<String>,
}

#[derive(Serialize)]
struct ResBody {
    tasks: Vec<models::ResTask>,
    query: Q,
}

pub async fn home(
    q: web::Query<Q>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res_body = web::block(move || {
        let conn = pool.get().unwrap();
        let query = q.into_inner();
        let res_tasks = query.config().query(&user, &conn)?;

        Ok(ResBody {
            tasks: res_tasks,
            query: query,
        })
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}

#[derive(Serialize)]
pub enum Config {
    Home,
    Leaves,
    Roots,
    Archives,
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
        use crate::schema::allocations::dsl::{allocations, owner};
        use crate::schema::tasks::dsl::{tasks, assign, is_archived, is_starred, updated_at};
        use crate::schema::users::dsl::users;

        let archives = if let Self::Archives = self { true } else { false };

        let _intermediate = tasks
            .filter(assign.eq(&user.id))
            .filter(is_archived.eq(&archives))
            .inner_join(users)
            .select(models::SelTask::columns());
        let res_tasks = if archives {
            _intermediate
            .order((is_starred.desc(), updated_at.desc()))
            .limit(100)
            .load::<models::SelTask>(conn)?
            .into_iter().map(|t| t.to_res()).collect()
        } else {
            let mut _res_tasks = _intermediate
                .order(updated_at.desc())
                .load::<models::SelTask>(conn)?
                .into_iter().map(|t| t.to_res()).collect();
            let arrows = models::Arrows::among(&_res_tasks, conn)?;
            arrows.set_lr(&mut _res_tasks);
            let _allocations = allocations
                .filter(owner.eq(&user.id))
                .select(models::Allocation::columns())
                .load::<models::Allocation>(conn)?;
            let mut sorter = Sorter {
                tasks: _res_tasks,
                allocations: _allocations,
                now: Utc::now(),
                tz: user.tz,
            };
            sorter.exec(arrows);
            sorter.tasks
        };
        Ok(res_tasks)
    }
}

struct Sorter {
    tasks: Vec<models::ResTask>,
    allocations: Vec<models::Allocation>,
    now: DateTime<Utc>,
    tz: Tz,
}

impl Sorter {
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
    fn allocations_set(&self, date0: Date<Utc>, days: i64) -> IntervalSet<i64> {
        (-1..=days+1).flat_map(|i| {
            self.allocations.iter().map(move |alc| {
                let open = date0.with_timezone(&self.tz).and_time(alc.open).unwrap() + Duration::days(i);
                let close = open + Duration::hours(alc.hours as i64);
                (open.timestamp(), close.timestamp())
            })
        }).collect::<Vec<(i64, i64)>>().to_interval_set()
    }
    fn splice(&self, dt: DateTime<Utc>) -> i64 {
        let mut days = dt.signed_duration_since(self.now).num_days();
        if dt < self.now { days -= 1 } // floor negative
        let daily = self.allocations.iter().map(|alc| alc.hours as i64).sum::<i64>() * 3600;
        let adjust = {
            let within_last_1 = (
                (self.now + Duration::days(days)).timestamp(),
                dt.timestamp(),
            ).to_interval_set();
            let allocations_set = self.allocations_set(self.now.date(), 0);
            within_last_1.intersection(&allocations_set).size() as i64
        };
        daily * days + adjust
    }
}

#[derive(Debug, PartialEq)]
struct SubSorter {
    cursor: i64,
    entries: Vec<i32>,
    arrows: models::Arrows,
    map: HashMap<i32, SubTask>,
}

#[derive(Debug, PartialEq)]
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


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t_110() {
        let task = SubTask {
            startable: None,
            deadline: Some(360),
            priority: None,
            weight: Some(120),
            rank: None,
        };
        let mut map = HashMap::new();
        map.insert(0, task);
        let mut sub = SubSorter {
            cursor: 0,
            entries: vec![0],
            arrows: models::Arrows {
                arrows: Vec::new(),
            },
            map: map,
        };
        sub.exec();
        assert_eq!(sub.map[&0], SubTask {
            startable: None,
            deadline: Some(360),
            priority: Some(-240),
            weight: Some(120),
            rank: Some(0),
        });
    }
}
