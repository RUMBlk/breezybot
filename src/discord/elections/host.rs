use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use chrono::NaiveDate;
use poise::serenity_prelude::{ CacheHttp, Message, Role, RoleId, Guild, CreateMessage };
use chrono;
use sea_orm::{DatabaseConnection, QuerySelect, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, IntoActiveModel };

use crate::Data;
use crate::database as db;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Operations {
    Assign,
    Remove,
}

pub async fn form_results (
    db: &DatabaseConnection,
    election: db::entities::elections::Model,
    guild: Guild,
    role: &Role,
) -> HashMap<Operations, HashSet<u64>> {
    let mut operations = HashMap::new();

    if let Ok(candidates) = db::queries::votes::sum_votes_for_election(election.id)
    .select_only()
    .column(db::entities::candidates::Column::User)
    .limit(election.limit as u64)
    .into_tuple::<String>()
    .all(db)
    .await {
            let candidates: HashSet<u64> = candidates.into_iter().filter_map(|s| s.parse::<u64>().ok()).collect();
            let role_holders: HashSet<u64> = guild.members
            .into_iter()
            .filter(|(_, member)| member.roles.contains(&role.id))
            .map(|(user_id, _)| user_id.as_u64().to_owned())
            .collect();

            // Remove roles from holders not in the election winners
            let remove_from: HashSet<u64> = role_holders.difference(&candidates).cloned().collect();
            if remove_from.len() > 0 {
                operations.insert(Operations::Remove, remove_from);
            }

            // Add roles to winners
            let add_to: HashSet<u64> = candidates.difference(&role_holders).cloned().collect();
            if add_to.len() > 0 {
                operations.insert(Operations::Assign, add_to);
            }
    }
    operations
}

pub async fn schedule_next(
    db: &DatabaseConnection,
    election: db::entities::elections::Model,
    now: NaiveDate,
) -> Option<NaiveDate> {
    if let Some(each) = election.each.clone() {
        let mut election = election.into_active_model();
        let schedule_for = match each.to_uppercase().as_str() {
            "DAY" => now.checked_add_days(chrono::Days::new(1)),
            "WEEK" => now.checked_add_days(chrono::Days::new(7)),
            "MONTH" => now.checked_add_months(chrono::Months::new(1)),
            "YEAR" => now.checked_add_months(chrono::Months::new(12)),
            _ => None,
        };
        election.next = sea_orm::Set(schedule_for);
        let _ = election.update(db).await;
        schedule_for
    } else { None }
}

pub async fn purge_candidate(db: &DatabaseConnection, user_id: u64) {
    if let Ok(model) = db::entities::candidates::Entity::find()
    .filter(db::entities::candidates::Column::User.eq(user_id))
    .one(db)
    .await {
        if let Some(candidate) = model {
            let candidate = candidate.into_active_model();
            let _ = db::entities::candidates::Entity::delete(candidate).exec(db).await;
        }
    }
}

pub async fn affected(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    if !message.author.bot {
        if let Some(db) = &data.db {
            if let Ok(elections) = db::entities::votes::Entity::find()
            .inner_join(db::entities::members::Entity)
            .inner_join(db::entities::candidates::Entity)
            .filter(db::entities::members::Column::User.eq(message.author.id.to_string()))
            .group_by(db::entities::candidates::Column::Election)
            .select_only()
            .column(db::entities::candidates::Column::Election)
            .into_tuple::<i64>()
            .all(db)
            .await {
                let mut transaction: HashMap<u64, HashMap<Operations, Vec<RoleId>>> = HashMap::new();
                let mut announcement: String = String::new();
                if let Some(guild) = message.guild(ctx.cache().unwrap()) {
                    let locale = db::queries::guilds::locale(db, &guild.id.to_string()).await;
                    for election_id in elections {
                        if let Ok(model) = db::entities::elections::Entity::find_by_id(election_id).one(db).await {
                            if let Some(election) = model {
                                if let Some(next) = election.next  {
                                    let now = chrono::Local::now().date_naive();
                                    if now >= next {
                                        if let Some(role) = guild.to_owned().roles.get(&RoleId::from(election.role.parse::<u64>().unwrap())) {
                                            let operations = form_results(db, election.clone(), guild.to_owned(), role).await;
                                            if operations.len() > 0 {
                                                if announcement.is_empty() { 
                                                    announcement += t!("elections.announcement.header", locale=&locale).deref();
                                                    announcement += "\n";
                                                };
                                                announcement += t!("elections.announcement.role", locale=&locale, role=role.id.to_string()).deref();
                                                announcement += "\n";
                                            }
                                            for (operation, user_ids) in operations {
                                                match operation {
                                                    Operations::Assign => { 
                                                        if user_ids.len() > 0 {
                                                            announcement += t!("elections.announcement.promoted", locale=&locale).deref();
                                                        }
                                                    },
                                                    Operations::Remove => {
                                                        if user_ids.len() > 0 {
                                                            announcement += t!("elections.announcement.demoted", locale=&locale).deref();
                                                        }
                                                    },    
                                                }
                                                for user_id in user_ids {
                                                    announcement += &format!("<@{}> ", user_id);
                                                    let member = transaction.entry(user_id).or_insert(HashMap::new());
                                                    let roles = member.entry(operation.clone()).or_insert(Vec::new());
                                                    roles.push(role.id);
                                                }
                                                announcement += "\n";
                                            }
                                            if let Some(date) = schedule_next(db, election, now).await {
                                                if !announcement.is_empty() { announcement += t!("elections.announcement.scheduled_for", locale=&locale, date=date.to_string()).deref() };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    for (user_id, operations) in transaction {
                        if let Ok(mut member) = guild.member(ctx.http(), user_id).await {
                            for (operation, roles) in operations {
                                if let Err(_) = match operation {
                                    Operations::Assign => {
                                        member.add_roles(ctx.http(), &roles).await
                                    },
                                    Operations::Remove => {
                                        member.remove_roles(ctx.http(), &roles).await
                                    },
                                } {
                                    announcement = t!("elections.host.lack_permissions", locale=&locale).to_string();
                                    break;
                                }
                            }
                        } else {
                            purge_candidate(db, user_id).await;
                        }
                    }

                    if !announcement.is_empty() {
                        if let Some(system_id) = guild.system_channel_id {
                            let _ = system_id.send_message(ctx.http(), move |r: &mut CreateMessage<'_>| -> &mut CreateMessage<'_> {
                                r
                                .content(announcement)
                            }).await;
                        }
                    }
                }
            }
        }
    }
}


