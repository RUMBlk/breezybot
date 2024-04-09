use chrono::NaiveDate;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{cache::FromStrAndCache, Role, RoleId};
use comfy_table::Table;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use crate::Context;

use sea_orm::{ EntityTrait, QueryFilter, ColumnTrait, QuerySelect};
use crate::database as db;

#[derive(Debug, poise::ChoiceParameter)]
pub enum TimePeriods {
    DAY,
    WEEK,
    MONTH,
    YEAR,
}

impl TimePeriods {
    pub fn to_datetime_from_now(&self) -> Option<NaiveDate> {
        let now = chrono::Local::now().date_naive();
        match self {
            TimePeriods::DAY => { now.checked_add_days(chrono::Days::new(1)) },
            TimePeriods::WEEK => { now.checked_add_days(chrono::Days::new(7)) },
            TimePeriods::MONTH => { now.checked_add_months(chrono::Months::new(1)) },
            TimePeriods::YEAR => { now.checked_add_months(chrono::Months::new(12)) },
        }
    }
}

pub async fn list(ctx: Context<'_>, db: &DatabaseConnection, locale: String, limit: Option<u8>) -> String {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let response = match db::entities::prelude::Elections::find()
    .filter(db::entities::elections::Column::Guild.eq(&guild_id))
    .limit(limit.unwrap_or(10) as u64)
    .all(db)
    .await {
        Ok(elections) => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL_CONDENSED)
                .set_header(vec![
                    t!("elections.roles.list.table.index", locale=&locale),
                    t!("elections.roles.list.table.name", locale=&locale),
                    t!("elections.roles.list.table.scheduled_for", locale=&locale),
                    t!("elections.roles.list.table.hosted_each", locale=&locale),
                ]);

            for election in elections {
                let mut index = 0;
                if let Some(role) = ctx.cache().guild_roles(ctx.guild_id().unwrap()).expect("").get(&RoleId::from_str(ctx.cache(), election.role.as_str()).unwrap()) {
                    index += 1;
                    let scheduled_for = match election.next {
                        Some(next) => next.to_string(),
                        None => "-".to_string(),
                    };
                    table.add_row(vec![
                        index.to_string(),
                        role.name.to_string(),
                        scheduled_for,
                        election.each.unwrap_or("-".to_string()),
                    ]);
                };
            };
            t!("elections.roles.list.success", locale=&locale, table=table )
        }
        Err(_) => t!("elections.roles.list.empty", locale=&locale)
    }
    .to_string();
    response
}

pub async fn add(ctx: Context<'_>, db: &DatabaseConnection, locale: String, role: Role, number_of_positions: Option<i16>, schedule: Option<TimePeriods>) -> String {
    let (next, each) = match schedule {
        Some(each) => { 
            (    
                each.to_datetime_from_now(),
                Some(each.to_string()),
            )
        },
        None => (None, None),
    };

    let response = match db::entities::elections::Entity::insert(
        db::entities::elections::ActiveModel {
            guild: sea_orm::Set(ctx.guild_id().unwrap().to_string().clone()),
            role: sea_orm::Set(role.id.to_string()),
            limit: sea_orm::Set(number_of_positions.unwrap_or(1) as i16),
            next: sea_orm::Set(next),
            each: sea_orm::Set(each),
            ..Default::default()
        },
    )
    .exec(db)
    .await
    {
        Ok(_) => { 
            t!("elections.roles.add.success", locale=&locale, role=role.name)
        },
        Err(_) => t!("elections.roles.add.exists", locale=&locale, role=role.name),
    }
    .to_string();
    response
}

pub async fn edit(db: &DatabaseConnection, locale: String, role: Role, number_of_positions: Option<i16>, schedule: Option<TimePeriods>) -> String {
   let dberr = t!("errors.database.oops", locale=&locale);
    let (next, period) = match schedule {
        Some(each) => { 
            (    
                each.to_datetime_from_now(),
                Some(each.to_string()),
            )
        },
        None => (None, None),
    };

    let response = match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await
    {
        Ok(model) => { 
            match model {
                Some(election) => {
                    let limit = number_of_positions.unwrap_or(election.limit);
                    let mut election = election.into_active_model();
                    election.limit = sea_orm::Set(limit.clone());
                    election.next = sea_orm::Set(next);
                    election.each = sea_orm::Set(period.clone());
                    match election.update(db).await {
                        Ok(_) => t!("elections.roles.edit.success", locale=&locale, role=role.name, number_of_positions=limit, period=period.unwrap_or("-".to_string())),
                        Err(_) => dberr,
                    }
                },
                None => t!("elections.roles.edit.not_found", locale=&locale, role=role.name),
            } 
        },
        Err(_) => dberr,
    }
    .to_string();
    response
}

pub async fn delete(db: &DatabaseConnection, locale: String, role: Role) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await {
        Ok(model) => { 
            match model {
                Some(row) => {
                    match db::entities::elections::Entity::delete(row.into_active_model()).exec(db).await {
                        Ok(_) => t!("elections.roles.delete.success", locale=&locale, role=role.name),
                        Err(_) => dberr,
                    }
                },
                None => t!("elections.roles.delete.not_found", locale=&locale, role=role.name), 
            }
        },
        Err(_) => dberr,
    }
    .to_string()
}
