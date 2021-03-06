use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use diesel::prelude::*;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::cmp::max;

use crate::errors;
use crate::models::{self, Selectable};
use crate::schema::{tasks, users};
use crate::utils;

#[derive(Deserialize)]
pub struct ReqBody {
    pub text: String,
}

#[derive(Serialize)]
enum ResBody {
    Command(ResCommand),
    Tasks {
        created: i32,
        updated: i32,
    },
}

pub async fn text(
    req: web::Json<ReqBody>,
    user: models::AuthedUser,
    pool: web::Data<models::Pool>,
) -> Result<HttpResponse, errors::ServiceError> {

    let req = req.into_inner().wash().parse::<Req>()?;

    let res_body = web::block(move || {
        let conn = pool.get().unwrap();
        match req {
            Req::Command(cmd) => {
                let res_command = match cmd {
                    // TODO /alias
                    ReqCommand::Help              => ResCommand::help(),
                    ReqCommand::User(request)     => request.handle(&user, &conn)?,
                    ReqCommand::Search(condition) => condition.extract(&user, &conn)?,
                    ReqCommand::Tutorial          => ResCommand::tutorial(),
                    ReqCommand::Coffee            => return Err(errors::ServiceError::BadRequest("I'm a teapot.".into())),
                };
                Ok(ResBody::Command(res_command))
            },
            Req::Tasks(tasks) => {
                Ok(tasks.read(&user)?.accept(&user, &conn)?.upsert(&conn)?)
            }
        }
    }).await?;

    Ok(HttpResponse::Ok().json(res_body))
}

#[derive(Debug, PartialEq)]
pub enum Req {
    Command(ReqCommand),
    Tasks(ReqTasks),
}

#[derive(Debug, PartialEq)]
pub enum ReqCommand {
    Help,
    User(ReqUser),
    Search(Condition),
    Tutorial,
    Coffee,
}

#[derive(Debug, PartialEq)]
pub enum ReqUser {
    Info,
    Modify(ReqModify),
}

#[derive(Debug, PartialEq)]
pub enum ReqModify {
    Email(String),
    Password(PasswordSet),
    Name(String),
    Timescale(Timescale),
    Allocations(Vec<ReqAllocation>),
}

#[derive(Debug, PartialEq)]
pub struct PasswordSet {
    pub old: String,
    pub new: String,
    pub confirmation: String,
}

pub type ReqAllocation = models::ResAllocation;

#[derive(Serialize)]
enum ResCommand {
    Help(String),
    User(ResUser),
    Search {
        tasks: Vec<models::ResTask>,
    },
    Tutorial {
        tasks: Vec<models::ResTask>,
    },
}

#[derive(Serialize)]
enum ResUser {
    Info {
        since: DateTime<Utc>,
        executed: i32,
        tz: Tz,
    },
    Modify(ResModify),
}

#[derive(Serialize)]
enum ResModify {
    Email(String),
    Password(()),
    Name(String),
    Timescale(String),
    Allocations(Vec<models::ResAllocation>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Timescale {
    Year,
    Quarter,
    Month,
    Week,
    Day,
    Hours,
    Hour,
    Minutes,
    Minute,
    Second,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Condition {
    pub boolean: Boolean,
    // TODO limit 333<#<777 use to !is_archived
    // TODO impl 333<!<777 for critical path
    // show critical Path between 2 selected tasks
    pub context: Range<i32>,
    pub weight: Range<f32>,
    pub startable: Range<models::EasyDateTime>,
    pub deadline: Range<models::EasyDateTime>,
    pub created_at: Range<models::EasyDateTime>,
    pub updated_at: Range<models::EasyDateTime>,
    pub title: Option<Expression>,
    pub assign: Option<Expression>,
    pub link: Option<Expression>,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Boolean {
    pub is_archived: Option<bool>,
    pub is_starred: Option<bool>,
    pub is_leaf: Option<bool>,
    pub is_root: Option<bool>,
}

type Range<T> = (Option<T>, Option<T>);

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Expression {
    Words(Vec<String>),
    Regex(String),
}

#[derive(Debug, Default, PartialEq)]
pub struct ReqTasks {
    pub tasks: Vec<ReqTask>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ReqTask {
    // indent #id joint] * TITLE startable- -deadline $weight @assign [joint link
    pub indent: i32,
    pub attribute: Attribute,
    pub link: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Attribute {
    pub is_starred: bool,
    pub id: Option<i32>,
    pub weight: Option<f32>,
    pub joint_head: Option<String>,
    pub joint_tail: Option<String>,
    pub assign: Option<String>,
    pub startable: Option<models::EasyDateTime>,
    pub deadline: Option<models::EasyDateTime>,
    pub title: String,
}

impl ResCommand {
    fn help() -> Self {
        Self::Help(String::from(
            // TODO search examples
            "\
            <!-- Press [Ctrl] + [↓] -->\n\
            \n\
            \n\
            \n\
            <!-- Select one, remove <!-- prefix, configure it, and send. -->\n\
            \n\
            <!-- / <!-- this help -->\n\
            <!-- /tutorial <!-- tutorial -->\n\
            <!-- /u <!-- show user info -->\n\
            <!-- /u -e {email} <!-- modify user email -->\n\
            <!-- /u -p {old} {new} {confirmation} <!-- modify user password -->\n\
            <!-- /u -n {name} <!-- modify user name -->\n\
            <!-- /u -t {timescale} <!-- modify user default timescale -->\n\
            <!-- /u -a {h}:{m}-{i}h {h}:{m}-{i}h ... <!-- modify user time allocations -->\n\
            <!-- /s {conditions} <!-- search for tasks by conditions -->\n\
            "
        ))
    }
    fn tutorial() -> Self {
        Self::Tutorial {
            tasks : vec![
                models::ResTask {
                    id: 0,
                    title: String::from("Press H to return home"),
                    assign: String::from("sprig"),
                    is_archived: false,
                    is_starred: true,
                    startable: None,
                    deadline: None,
                    priority: None,
                    weight: None,
                    link: None, // TODO tutorial external
                    schedule: None,
                },
            ],
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "users"]
struct AltUser {
    email: Option<String>,
    hash: Option<String>,
    name: Option<String>,
    timescale: Option<String>,
}

impl ReqUser {
    fn handle(self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<ResCommand, errors::ServiceError> {
        let res = match self {
            Self::Info => self.info(user, conn)?,
            Self::Modify(req) => ResUser::Modify(req.exec(user, conn)?),
        };
        Ok(ResCommand::User(res))
    }
    fn info(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<ResUser, errors::ServiceError> {
        use crate::schema::tasks::dsl::{tasks, assign, is_archived};
        use crate::schema::users::dsl::{users, created_at};

        let since = users.find(user.id).select(created_at).first::<DateTime<Utc>>(conn)?;
        let executed = tasks
        .filter(assign.eq(&user.id))
        .filter(is_archived)
        .count().get_result::<i64>(conn)? as i32;

        Ok(ResUser::Info {
            since: since,
            executed: executed,
            tz: user.tz,
        })
    }
}

impl ReqModify {
    fn exec(self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<ResModify, errors::ServiceError> {
        use diesel::dsl::{select, exists};
        use crate::schema::users::dsl::{users, email, name};
        use crate::schema::allocations::dsl::{allocations, owner};

        if let Self::Allocations(req_alcs) = self {
            let mut ins = Vec::new();
            for alc in &req_alcs {
                ins.push(alc.verify(user)?);
            }
            diesel::delete(allocations.filter(owner.eq(&user.id))).execute(conn)?;
            diesel::insert_into(allocations).values(&ins).execute(conn)?;
            return Ok(ResModify::Allocations(req_alcs))
        }
        let mut alt_user = AltUser {
            email: None,
            hash: None,
            name: None,
            timescale: None,
        };
        let res = match self {
            Self::Email(s) => {
                if select(exists(users.filter(email.eq(&s)))).get_result(conn)? {
                    return Err(errors::ServiceError::BadRequest(format!(
                        "email already in use: {}",
                        s,
                    )))
                }
                alt_user.email = Some(s.clone());
                ResModify::Email(s)
            },
            Self::Password(password_set) => {
                let hash = password_set.verify(user, conn)?;
                alt_user.hash = Some(hash);
                ResModify::Password(())
            },
            Self::Name(s) => {
                if select(exists(users.filter(name.eq(&s)))).get_result(conn)? {
                    return Err(errors::ServiceError::BadRequest(format!(
                        "username already in use: {}",
                        s,
                    )))
                }
                alt_user.name = Some(s.clone());
                ResModify::Name(s)
            },
            Self::Timescale(timescale) => {
                alt_user.timescale = Some(timescale.as_str().into());
                ResModify::Timescale(timescale.as_str().into())
            },
            _ => unreachable!(),
        };
        diesel::update(user).set(&alt_user).execute(conn)?;

        Ok(res)
    }
}

impl PasswordSet {
    fn verify(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<String, errors::ServiceError> {
        use crate::schema::users::dsl::users;

        let min_password_len = 8;
        let old_hash = users.find(user.id).first::<models::User>(conn)?.hash;
        if utils::verify(&old_hash, &self.old)? {
            if min_password_len <= self.new.len() {
                if self.new == self.confirmation {
                    let new_hash = utils::hash(&self.new)?;
                    return Ok(new_hash)
                }
                return Err(errors::ServiceError::BadRequest(format!(
                    "new password mismatched with confirmation.",
                )))
            }
            return Err(errors::ServiceError::BadRequest(format!(
                "password should be at least {} length.",
                min_password_len,
            )))
        }
        return Err(errors::ServiceError::BadRequest(format!(
            "current password seems to be wrong.",
        )))
    }
}

impl Timescale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Year => "Y",
            Self::Quarter => "Q",
            Self::Month => "M",
            Self::Week => "W",
            Self::Day => "D",
            Self::Hours => "6h",
            Self::Hour => "h",
            Self::Minutes => "15m",
            Self::Minute => "m",
            Self::Second => "s",
        }
    }
}

impl ReqAllocation {
    fn verify(&self,
        user: &models::AuthedUser,
    ) -> Result<models::Allocation, errors::ServiceError> {
        if let Some(time) = NaiveTime::from_hms_opt(self.open_h as u32, self.open_m as u32, 0) {
            if (1..=24).contains(&self.hours) {
                return Ok(models::Allocation {
                    owner: user.id,
                    open: time,
                    hours: self.hours,
                })
            }
            return Err(errors::ServiceError::BadRequest("please specify 1 to 24 hours.".into()))
        }
        Err(errors::ServiceError::BadRequest("time notation invalid.".into()))
    }
}

impl Condition {
    fn extract(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<ResCommand, errors::ServiceError> {
        use crate::schema::arrows::dsl::arrows;

        let mut res_tasks = self.query(user, conn)?;
        self.filter_regex(&mut res_tasks)?;
        if max(self.context.0, self.context.1).is_some() {
            // TODO load all arrows ?
            let _arrows: models::Arrows = arrows.load::<models::Arrow>(conn)?.into();
            self.filter_context(&mut res_tasks, &_arrows);
        }
        Ok(ResCommand::Search {
            tasks: res_tasks,
        })
    }
    fn query(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<Vec<models::ResTask>, errors::ServiceError> {
        use diesel::dsl::exists;
        use crate::schema::arrows::dsl::*;
        use crate::schema::permissions::dsl::*;
        use crate::schema::tasks::dsl::*;
        use crate::schema::users::dsl::{users, name};

        let mut query = tasks
        .filter(exists(permissions
            .filter(subject.eq(&user.id))
            .filter(object.eq(assign))
        ))
        .inner_join(users)
        .select(models::SelTask::columns())
        .into_boxed();

        if let Some(b) = &self.boolean.is_archived {
            query = query.filter(is_archived.eq(b))
        }
        if let Some(b) = &self.boolean.is_starred {
            query = query.filter(is_starred.eq(b))
        }
        if let Some(b) = &self.boolean.is_leaf {
            query = query.filter(
                exists(arrows.filter(target.eq(id))).eq(!b)
            )
        }
        if let Some(b) = &self.boolean.is_root {
            query = query.filter(
                exists(arrows.filter(source.eq(id))).eq(!b)
            )
        }
        if let Some(w) = &self.weight.0 {
            query = query.filter(weight.ge(w))
        }
        if let Some(w) = &self.weight.1 {
            query = query.filter(weight.le(w))
        }
        if let Some(dt) = &self.startable.0 {
            query = query.filter(startable.ge(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.startable.1 {
            query = query.filter(startable.le(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.deadline.0 {
            query = query.filter(deadline.ge(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.deadline.1 {
            query = query.filter(deadline.le(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.created_at.0 {
            query = query.filter(created_at.ge(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.created_at.1 {
            query = query.filter(created_at.le(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.updated_at.0 {
            query = query.filter(updated_at.ge(user.globalize(&dt)?))
        }
        if let Some(dt) = &self.updated_at.1 {
            query = query.filter(updated_at.le(user.globalize(&dt)?))
        }
        if let Some(Expression::Words(words)) = &self.title {
            for w in words {
                query = query.filter(title.like(format!("%{}%", w)))
            }
        }
        if let Some(Expression::Words(words)) = &self.assign {
            for w in words {
                query = query.filter(name.like(format!("%{}%", w)))
            }
        }
        if let Some(Expression::Words(words)) = &self.link {
            for w in words {
                query = query.filter(link.like(format!("%{}%", w)))
            }
        }
        Ok(query
            .order((is_starred.desc(), updated_at.desc()))
            .limit(100) // TODO limit extraction ?
            .load::<models::SelTask>(conn)?
            .into_iter().map(|t| t.to_res()).collect()
        )
    }
    fn filter_regex(&self,
        tasks: &mut Vec<models::ResTask>,
    ) -> Result<(), errors::ServiceError> {
        if let Some(Expression::Regex(regex)) = &self.title {
            let regex = Regex::new(&regex)?;
            tasks.retain(|t| regex.is_match(&t.title))
        }
        if let Some(Expression::Regex(regex)) = &self.assign {
            let regex = Regex::new(&regex)?;
            tasks.retain(|t| regex.is_match(&t.assign))
        }
        if let Some(Expression::Regex(regex)) = &self.link {
            let regex = Regex::new(&regex)?;
            tasks.retain(|t| regex.is_match(&**t.link.as_ref().unwrap_or(&String::new())));
        }
        Ok(())
    }
    fn filter_context(&self,
        tasks: &mut Vec<models::ResTask>,
        arrows: &models::Arrows,
    ) {
        if let Some(id) = self.context.0 {
            let ids = models::Tid::from(id).nodes_to(models::LR::Root, arrows);
            tasks.retain(|t| ids.iter().any(|id| *id == t.id))
        }
        if let Some(id) = self.context.1 {
            let ids = models::Tid::from(id).nodes_to(models::LR::Leaf, arrows);
            tasks.retain(|t| ids.iter().any(|id| *id == t.id))
        }
    }
}

struct Acceptor {
    tasks: Vec<TmpTask>,
    arrows: TmpArrows,
}

type TmpArrows =  models::Arrows;

struct TmpTask {
    id: Option<i32>,
    title: String,
    assign: Option<String>,
    is_starred: bool,
    startable: Option<DateTime<Utc>>,
    deadline: Option<DateTime<Utc>>,
    weight: Option<f32>,
    link: Option<String>,
}

impl ReqTasks {
    fn read(self,
        user: &models::AuthedUser,
    ) -> Result<Acceptor, errors::ServiceError> {
        let iter =  self.tasks.iter().enumerate().rev();
        let mut tmp_arrows = Vec::new();
        for (src, t) in iter.clone() {
            if let Some((tgt, _)) = iter.clone()
            .filter(|(idx, _)| *idx < src)
            .find(|(_, _t)| _t.indent < t.indent) {
                tmp_arrows.push(models::Arrow {
                    source: src as i32,
                    target: tgt as i32,
                });
            }
            for (tgt, _) in iter.clone().filter(|(_, _t)| {
                if let (Some(tail), Some(head)) = (&_t.attribute.joint_tail, &t.attribute.joint_head) {
                    tail == head
                } else { false }
            }) {
                tmp_arrows.push(models::Arrow {
                    source: src as i32,
                    target: tgt as i32,
                });
            }
        }
        let mut tmp_tasks = Vec::new();
        for t in self.tasks {
            let mut startable = None;
            if let Some(dt) = t.attribute.startable {
                startable = Some(user.globalize(&dt)?)
            }
            let mut deadline = None;
            if let Some(dt) = t.attribute.deadline {
                deadline = Some(user.globalize(&dt)?)
            }
            tmp_tasks.push(TmpTask {
                id: t.attribute.id,
                title: t.attribute.title,
                assign: t.attribute.assign,
                is_starred: t.attribute.is_starred,
                startable: startable,
                deadline: deadline,
                weight: t.attribute.weight,
                link: t.link,
            })
        }
        Ok(Acceptor {
            tasks: tmp_tasks,
            arrows: tmp_arrows.into(),
        })
    }
}

struct Upserter {
    tasks: Vec<TmpTaskOk>,
    arrows: TmpArrows,
}

struct TmpTaskOk {
    id: Option<i32>,
    title: String,
    assign: i32,
    is_starred: bool,
    startable: Option<DateTime<Utc>>,
    deadline: Option<DateTime<Utc>>,
    weight: Option<f32>,
    link: Option<String>,
}

impl Acceptor {
    fn accept(self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<Upserter, errors::ServiceError> {

        self.no_loop()?;
        self.valid_sd()?;
        self.valid_tid_use()?;
        self.valid_tid(user, conn)?;
        let assigns = self.valid_assign(user, conn)?;

        let tasks = self.tasks.into_iter().zip(assigns.iter()).map(|(t, &a)| TmpTaskOk {
            id: t.id,
            title: t.title,
            assign: a,
            is_starred: t.is_starred,
            startable: t.startable,
            deadline: t.deadline,
            weight: t.weight,
            link: t.link,
        }).collect::<Vec<TmpTaskOk>>();

        Ok(Upserter {
            tasks: tasks,
            arrows: self.arrows,
        })
    }
    fn no_loop(&self) -> Result<(), errors::ServiceError> {
        if self.arrows.has_cycle() {
            return Err(errors::ServiceError::BadRequest("loop found.".into()))
        }
        Ok(())
    }
    fn valid_sd(&self) -> Result<(), errors::ServiceError> {
        if let Some(t) = self.tasks.iter()
        .filter(|t| t.deadline.is_some() && t.startable.is_some())
        .find(|t| t.deadline.unwrap() < t.startable.unwrap()) {
            return Err(errors::ServiceError::BadRequest(format!(
                "{}... deadline then startable.",
                t.title.chars().take(8).collect::<String>(),
            )))
        }
        Ok(())
    }
    fn valid_tid_use(&self) -> Result<(), errors::ServiceError> {
        self.tid_unique()?;
        for path in self.arrows.paths() {
            self.tid_single_by(&path)?;
        }
        Ok(())
    }
    fn tid_unique(&self) -> Result<(), errors::ServiceError> {
        let mut ids = self.ids();
        ids.sort();
        let mut last = i32::MIN;
        for id in ids {
            if id == last {
                return Err(errors::ServiceError::BadRequest(format!(
                    "#{} appears multiple times.",
                    id,
                )))
            }
            last = id
        }
        Ok(())
    }
    fn ids(&self) -> Vec<i32> {
        self.tasks.iter().filter_map(|t| t.id).collect::<Vec<i32>>()
    }
    fn tid_single_by(&self, path: &models::Path) -> Result<(), errors::ServiceError> {
        let ids = path.iter().filter_map(|idx| self.tasks.get(*idx as usize).unwrap().id).collect::<Vec<i32>>();
        if 1 < ids.len() {
            return Err(errors::ServiceError::BadRequest(format!(
                "#{} -> #{} existing nodes wiring.",
                ids.get(0).unwrap(),
                ids.get(1).unwrap(),
            )))
        }
        Ok(())
    }
    fn valid_tid(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<(), errors::ServiceError> {
        use diesel::dsl::exists;
        use crate::schema::permissions::dsl::*;
        use crate::schema::tasks::dsl::{tasks, assign};

        for id in self.ids() {
            if tasks
            .find(id)
            .filter(exists(permissions
                .filter(subject.eq(&user.id))
                .filter(object.eq(assign))
                .filter(edit)
            ))
            .first::<models::Task>(conn)
            .is_err() {
                return Err(errors::ServiceError::BadRequest(format!(
                    "#{}: item not found, or no edit permission.",
                    id,
                )))
            }
        }
        Ok(())
    }
    fn valid_assign(&self,
        user: &models::AuthedUser,
        conn: &models::Conn,
    ) -> Result<Vec<i32>, errors::ServiceError> {
        use diesel::dsl::exists;
        use crate::schema::permissions::dsl::*;
        use crate::schema::users::dsl::{users, id, name};

        let mut assigns = Vec::new();
        for t in &self.tasks {
            let mut assign = user.id;
            if let Some(_name) = &t.assign {
                match users
                .filter(name.eq(&_name))
                .filter(exists(permissions
                    .filter(subject.eq(&user.id))
                    .filter(object.eq(id))
                    .filter(edit)
                ))
                .first::<models::User>(conn) {
                    Ok(someone) => assign = someone.id,
                    Err(_) => {
                        return Err(errors::ServiceError::BadRequest(format!(
                            "@{}: user not found.",
                            _name,
                        )))
                    },
                }
            }
            assigns.push(assign)
        }
        Ok(assigns)
    }
}

#[derive(Insertable)]
#[table_name = "tasks"]
struct NewTask {
    title: String,
    assign: i32,
    is_starred: bool,
    startable: Option<DateTime<Utc>>,
    deadline: Option<DateTime<Utc>>,
    weight: Option<f32>,
    link: Option<String>,
}

#[derive(AsChangeset)]
#[table_name = "tasks"]
struct AltTask {
    title: Option<String>,
    assign: Option<i32>,
    is_starred: Option<bool>,
    startable: Option<Option<DateTime<Utc>>>,
    deadline: Option<Option<DateTime<Utc>>>,
    weight: Option<Option<f32>>,
    link: Option<Option<String>>,
}

impl Upserter {
    fn upsert(mut self,
        conn: &models::Conn,
    ) -> Result<ResBody, errors::ServiceError> {
        use crate::schema::arrows::dsl::arrows;
        use crate::schema::tasks::dsl::tasks;

        let mut permanents = Vec::new();
        let mut created = 0;
        let mut updated = 0;
        for t in self.tasks.into_iter() {
            let id = match t.id {
                None => {
                    let id = diesel::insert_into(tasks).values(&NewTask::from(t)).get_result::<models::Task>(conn)?.id;
                    created += 1;
                    id
                },
                Some(id) => {
                    diesel::update(tasks.find(id)).set(&AltTask::from(t)).execute(conn)?;
                    updated += 1;
                    id
                },
            };
            permanents.push(id)
        }
        for arw in &mut self.arrows.arrows {
            arw.source = *permanents.get(arw.source as usize).unwrap();
            arw.target = *permanents.get(arw.target as usize).unwrap();
        }
        diesel::insert_into(arrows).values(&self.arrows.arrows).execute(conn)?;

        Ok(ResBody::Tasks {
            created: created,
            updated: updated,
        })
    }
}

impl From<TmpTaskOk> for NewTask {
    fn from(tmp: TmpTaskOk) -> Self {
        Self {
            title: tmp.title,
            assign: tmp.assign,
            is_starred: tmp.is_starred,
            startable: tmp.startable,
            deadline: tmp.deadline,
            weight: tmp.weight,
            link: tmp.link,
        }
    }
}

impl From<TmpTaskOk> for AltTask {
    fn from(tmp: TmpTaskOk) -> Self {
        Self {
            title: Some(tmp.title),
            assign: Some(tmp.assign),
            is_starred: Some(tmp.is_starred),
            startable: Some(tmp.startable),
            deadline: Some(tmp.deadline),
            weight: Some(tmp.weight),
            link: Some(tmp.link),
        }
    }
}
