use sea_orm::*;
use chrono::{ NaiveDate, Days, Months };
use crate::database as db;
use db::entities::{ *, prelude::* };

#[derive(Clone, Copy)]
pub enum Schedule {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl ToString for Schedule {
    fn to_string(&self) -> String {
        match self {
            Self::Daily => String::from("Daily"),
            Self::Weekly => String::from("Weekly"),
            Self::Monthly => String::from("Monthly"),
            Self::Yearly => String::from("Yearly"),
        }
    }
}

impl Schedule {
    pub fn next_date(&self, date: Option<NaiveDate>) -> Option<NaiveDate> {
        let date = date.unwrap_or(chrono::Local::now().date_naive());
        match self {
            Self::Daily => date.checked_add_days(Days::new(1)),
            Self::Weekly => date.checked_add_days(Days::new(7)),
            Self::Monthly => date.checked_add_months(Months::new(1)),
            Self::Yearly => date.checked_add_months(Months::new(12)),
        }
    }
}

impl From<String> for Schedule {
    fn from(value: String) -> Self {
        match value.to_uppercase().as_str() {
            "DAILY" => Schedule::Daily,
            "WEEKLY" => Schedule::Weekly,
            "MONTHLY" => Schedule::Monthly,
            "YEARLY" => Schedule::Yearly,
            _ => panic!("Invalid schedule type"),
        }
    }
}

pub async fn exists(db: &DatabaseConnection, role: u64) -> bool {
    Elections::find().filter(elections::Column::Role.eq(role)).count(db).await.unwrap_or(0) > 0
}

pub async fn get(db: &DatabaseConnection, id: u64) -> Result<Option<elections::Model>, DbErr> {
    Elections::find_by_id(id).one(db).await
}

pub async fn create(db: &DatabaseConnection, role: u64, limit: i16, schedule: Option<Schedule>) -> Result<elections::ActiveModel, DbErr> {
    elections::ActiveModel {
        role: Set(role),
        limit: Set(limit),
        schedule: Set(schedule.and_then(|v| { Some(v.to_string().to_uppercase()) })),
        next: Set(schedule.and_then(|v| { v.next_date(None) })),
        ..Default::default()
    }.save(db).await
}

pub async fn delete(db: &DatabaseConnection, role: u64) -> Result<DeleteResult, DbErr> {
    Elections::delete_by_id(role).exec(db).await
}

pub async fn model_schedule_next(
    db: &DatabaseConnection,
    model: elections::Model,
) -> Result<elections::Model, DbErr> {
    let Some(ref schedule) = model.schedule else { return Err(DbErr::RecordNotUpdated) };
    let schedule_for = Schedule::from(schedule.to_owned()).next_date(None);
    
    let mut active_model = model.clone().into_active_model();
    active_model.next = sea_orm::Set(schedule_for);
    active_model.update(db).await
}