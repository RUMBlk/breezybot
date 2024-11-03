use sea_orm::{ *, prelude::Decimal };
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
    Elections::find().filter(elections::Column::Role.eq(role))
        .select_only()
        .column(elections::Column::Role)
        .into_tuple::<Decimal>()
        .one(db)
        .await
        .expect("")
        .is_some()
}

pub async fn get(db: &DatabaseConnection, id: u64) -> Result<Option<elections::Model>, DbErr> {
    Elections::find_by_id(id).one(db).await
}

pub async fn get_many(db: &DatabaseConnection, guild: u64, limit: u64) -> Result<Vec<elections::Model>, DbErr> {
    Elections::find().filter(elections::Column::Guild.eq(guild)).limit(limit).all(db).await
}

pub async fn create(db: &DatabaseConnection, role: u64, guild: u64, limit: Option<i16>, schedule: Option<Schedule>) -> Result<elections::Model, DbErr> {
    let mut model = elections::ActiveModel {
        role: Set(role.into()),
        guild: Set(guild.into()),
        schedule: Set(schedule.and_then(|v| { Some(v.to_string().to_uppercase()) })),
        scheduled_date: Set(schedule.and_then(|v| { v.next_date(None) })),
        ..Default::default()
    };
    limit.inspect(|v| model.limit = Set(*v));

    model.insert(db).await
}

pub async fn update(db: &DatabaseConnection, role: u64, limit: Option<i16>, schedule: Option<Schedule>) -> Result<elections::Model, DbErr> {
    let mut model = elections::ActiveModel {
        role: Set(role.into()),
        ..Default::default()
    };
    limit.inspect(|v| model.limit = Set(*v));
    if let Some(schedule) = schedule {
        let scheduled_date = Elections::find_by_id(role)
            .select_only()
            .column(elections::Column::ScheduledDate)
            .into_tuple::<NaiveDate>()
            .one(db) 
            .await
            .expect("");

        model.schedule = Set(Some(schedule.to_string().to_uppercase()));
        model.scheduled_date = Set(schedule.next_date(scheduled_date))
    }

    model.update(db).await
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
    active_model.scheduled_date = sea_orm::Set(schedule_for);
    active_model.update(db).await
}

/*pub async fn affected_elections_by_user(db: &DatabaseConnection, guild: u64, user: u64) -> Result<Vec<elections::Model>, sea_orm::DbErr> {
    Elections::find()
        .join_as(
            sea_orm::JoinType::InnerJoin,
            candidates::Relation::Votes.def(),
            Alias::new("votes"),
        )
        .inner_join(Candidates)
        .join_as(
            sea_orm::JoinType::InnerJoin,
            votes::Relation::Members.def(),
            Alias::new("members"),
        )
        .filter(members::Column::User.eq(user))
        .filter(elections::Column::Guild.eq(guild))
        .group_by(candidates::Column::Elections)
        .select_only()
        .columns(vec![
            elections::Column::Role,
            elections::Column::Guild,
            elections::Column::ScheduledDate,
            elections::Column::Schedule,
        ])
        .into_model::<elections::Model>()
        .all(db)
        .await
}*/