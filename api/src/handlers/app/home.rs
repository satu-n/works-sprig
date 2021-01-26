use actix_web::{web, HttpResponse};
use chrono::{Date, DateTime, Duration, NaiveDateTime, Utc};
use chrono_tz::Tz;
use diesel::prelude::*;
use gcollections::ops::{Bounded, Cardinality, Intersection};
use interval::interval_set::ToIntervalSet;
use interval::interval_set::{IntervalSet};
use serde::{Serialize, Deserialize};
use std::cmp::{max, min};
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
}

pub async fn home(
    q: web::Query<Q>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let res_body = web::block(move || {
        let conn = pool.get().unwrap();
        let res_tasks = q.into_inner().config().query(&user, &conn)?;

        Ok(ResBody {
            tasks: res_tasks,
        })
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}

#[derive(Serialize, Eq, PartialEq)]
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

        let is_archives = *self == Self::Archives;
        let _intermediate = tasks
            .filter(assign.eq(&user.id))
            .filter(is_archived.eq(&is_archives))
            .inner_join(users)
            .select(models::SelTask::columns());
        if is_archives {
            return Ok(
                _intermediate
                .order((is_starred.desc(), updated_at.desc()))
                .limit(100)
                .load::<models::SelTask>(conn)?
                .into_iter().map(|t| t.to_res()).collect()
            )
        }
        let mut res_tasks = _intermediate
            .order(updated_at.desc())
            .load::<models::SelTask>(conn)?
            .into_iter().map(|t| t.to_res()).collect();
        let arrows = models::Arrows::among(&res_tasks, conn)?;
        let _allocations = allocations
            .filter(owner.eq(&user.id))
            .select(models::Allocation::columns())
            .load::<models::Allocation>(conn)?;
        let sorter = Sorter {
            allocations: _allocations,
            now: Utc::now(),
            tz: user.tz,
        };
        sorter.exec(&mut res_tasks, arrows.clone());
        self.filter(&mut res_tasks, &arrows);
        Ok(res_tasks)
    }
    fn filter(&self, tasks: &mut Vec<models::ResTask>, arrows: &models::Arrows) {
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
    allocations: Vec<models::Allocation>,
    now: DateTime<Utc>,
    tz: Tz,
}

impl Sorter {
    fn exec(&self, tasks: &mut Vec<models::ResTask>, arrows: models::Arrows) {
        let mut sub = self.to_sub(tasks, arrows);
        sub.exec();
        // set priority
        for t in tasks.iter_mut() {
            if let Some(p) = sub.map[&t.id].priority {
                t.priority = Some(p as f32 / 3600.0) // hours from seconds
            }
        }
        if 0 < self.daily() {
            // set schedule
            for t in tasks.iter_mut() {
                if let (Some(l), Some(r)) = (sub.map[&t.id].startable, sub.map[&t.id].deadline) {
                    t.schedule = Some(models::Schedule {
                        l: self.unsplice(l).unwrap(),
                        r: self.unsplice(r).unwrap(),
                    })
                }
            }
        }
        tasks.sort_by(|a, b| sub.map[&a.id].rank.cmp(&sub.map[&b.id].rank));
        tasks.sort_by(|a, b| b.is_starred.cmp(&a.is_starred));
    }
    fn to_sub(&self, tasks: &Vec<models::ResTask>, arrows: models::Arrows) -> SubSorter {
        let mut map = HashMap::new();
        for t in tasks {
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
    fn daily(&self) -> i64 {
        self.allocations.iter().map(|alc| alc.hours as i64).sum::<i64>() * 3600
    }
    fn splice(&self, dt: DateTime<Utc>) -> i64 {
        let mut days = dt.signed_duration_since(self.now).num_days();
        if dt < self.now { days -= 1 } // floor negative
        let adjust = {
            let approx = self.now + Duration::days(days);
            let allocations_set = self.allocations_set(approx.date(), 0);
            (approx.timestamp(), dt.timestamp()).to_interval_set()
            .intersection(&allocations_set).size() as i64
        };
        self.daily() * days + adjust
    }
    fn unsplice(&self, dt: i64) -> Option<DateTime<Utc>> {
        let daily = self.daily();
        if  daily == 0 { return None }
        let approx = self.now + Duration::days(dt / daily);
        let mut remain = dt % daily;
        let mut cursor = approx.timestamp();
        for alc in self.allocations_set(approx.date(), 0) {
            if alc.upper() < cursor { continue }
            let point = max(alc.lower(), cursor);
            let draw =  min(alc.upper() - point, remain);
            cursor = point + draw;
            remain -= draw;
            if remain == 0 { break }
        }
        Some(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(cursor, 0), Utc))
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
    rank: Option<usize>,
}

struct Player {
    id: i32,
    priority: Option<i64>,
}

impl SubSorter {
    fn exec(&mut self) {
        let mut rank = 0;
        while !self.entries.is_empty() {
            if let Some(win) = self.winner() {
                let weight = self.map[&win.id].weight.unwrap_or_default();
                let edit = self.map.get_mut(&win.id).unwrap();
                edit.priority = win.priority;
                edit.rank = Some(rank);
                rank += 1;
                edit.startable = Some(self.cursor);
                self.cursor += weight;
                edit.deadline = Some(self.cursor);
                self.entries.retain(|id| *id != win.id);
                self.arrows.arrows.retain(|arw| arw.source != win.id);
            } else {
                self.cursor += 1;
            }
        }
    }
    fn winner(&self) -> Option<Player> {
        self.startables().into_iter().map(|id| Player {
            id: id,
            priority: self.priority(id),
        }).max_by_key(|player| player.priority)
    }
    fn startables(&self) -> Vec<i32> {
        self.entries.iter().copied()
        .filter(|id| self.map[&id].startable.map(|t| t <= self.cursor).unwrap_or(true))
        .filter(|id| models::Tid::from(*id).is(models::LR::Leaf, &self.arrows))
        .collect::<Vec<i32>>()
    }
    fn priority(&self, id: i32) -> Option<i64> {
        self.paths(id).iter()
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
    fn paths(&self, id: i32) -> Vec<models::Path> {
        let mut paths = models::Tid::from(id).paths_to(models::LR::Root, &self.arrows);
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
            startable: Some(0),
            deadline: Some(120),
            priority: Some(-240),
            weight: Some(120),
            rank: Some(0),
        });
    }
}
