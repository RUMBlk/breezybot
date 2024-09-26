use std::collections::HashMap;
use std::collections::HashSet;

use chrono::NaiveDate;
use poise::serenity_prelude::Member;
use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::{ User, CacheHttp, Message, Role, RoleId, Guild, CreateMessage, ChannelId, SerenityError };
use chrono;
use sea_orm::{DatabaseConnection, QuerySelect, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, IntoActiveModel };

use crate::database::queries::guilds::locale;
use crate::localization::Translations;
use crate::Data;
use crate::database as db;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Operations {
    Assign,
    Remove,
}

impl Operations {
    pub async fn apply<'a>(&self, ctx: &'a poise::serenity_prelude::prelude::Context, member: &'a mut Member, role_id: &RoleId)
    -> Result<(), SerenityError> {
        match &self {
            Operations::Assign => { 
                member.add_role(ctx.http(), role_id).await
            },
            Operations::Remove => {
                member.remove_role(ctx.http(), role_id).await
            }, 
        }?;
        Ok(())
    }
}

pub async fn form_results (
    db: &DatabaseConnection,
    election: &db::entities::elections::Model,
    guild: Guild,
    role: &Role,
) -> HashMap<Operations, HashSet<u64>> {
    let mut operations = HashMap::new();

    let Ok(candidates) = db::queries::votes::sum_votes_for_election(election.id)
        .select_only()
        .column(db::entities::candidates::Column::User)
        .limit(election.limit as u64)
        .into_tuple::<String>()
        .all(db)
        .await else { return operations };

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

    operations
}

pub async fn schedule_next(
    db: &DatabaseConnection,
    election: &db::entities::elections::Model,
    now: NaiveDate,
) -> Option<NaiveDate> {
    let mut election = election.clone().into_active_model();
    let schedule_for = match election.each.clone().unwrap()?.to_uppercase().as_str() {
        "DAY" => now.checked_add_days(chrono::Days::new(1)),
        "WEEK" => now.checked_add_days(chrono::Days::new(7)),
        "MONTH" => now.checked_add_months(chrono::Months::new(1)),
        "YEAR" => now.checked_add_months(chrono::Months::new(12)),
        _ => None,
    };
    election.next = sea_orm::Set(schedule_for);
    let _ = election.update(db).await;
    schedule_for
}

pub async fn purge_candidate(db: &DatabaseConnection, user_id: u64) {
    let Ok(Some(candidate)) = db::entities::candidates::Entity::find()
        .filter(db::entities::candidates::Column::User.eq(user_id))
        .one(db)
        .await else { return };
    let candidate = candidate.into_active_model();
    let _ = db::entities::candidates::Entity::delete(candidate).exec(db).await;
}

pub enum HostError {
    NotScheduledOrForced,
    ScheduledForLater,
    NoRoleId,
    RoleUnavailable,
}

pub async fn host(
    ctx: &poise::serenity_prelude::prelude::Context,
    db: &DatabaseConnection,
    guild: &Guild,
    election: &db::entities::elections::Model,
    force: bool,
) -> Result<(HashMap<Operations, HashSet<User>>, Option<NaiveDate>), HostError> {
    let now = chrono::Local::now().date_naive();
    if !force {
        let Some(next) = election.next else { return Err(HostError::NotScheduledOrForced) };
        if !(now >= next) { return Err(HostError::ScheduledForLater)};
    }
    let Some(role_id) = election.role.parse::<u64>()
        .ok()
        .and_then(|value| { Some(RoleId::from(value)) })
        else { return Err(HostError::NoRoleId) };
    let Some(role) = guild.roles.get(&role_id) else { return Err(HostError::RoleUnavailable) };
    let mut result = HashMap::new();
    let operations = form_results(db, election, guild.to_owned(), role).await;
    for (operation, user_ids) in operations {
        for user_id in user_ids.to_owned() {
            let Ok(mut member) = guild.member(ctx.http(), user_id).await else { purge_candidate(db, user_id).await; continue };
            let Ok(_) = operation.apply(ctx, &mut member, &role_id).await else { continue };
            result.entry(operation.clone()).or_insert(HashSet::new()).insert(member.user);
        }
    }

    Ok((result, schedule_next(db, election, now).await))
}

pub fn compile_announcement<'a>(
    tr: &Translations,
    locale: Option<&'a str>,
    operations: HashMap<Operations, HashSet<User>>,
    scheduled_for: Option<NaiveDate>,
    role_id: String,
) -> String {
    let mut announcement: String = String::new();
    if operations.len() > 0 {
        if announcement.is_empty() { 
            announcement = format!("{}\n", crate::loc!(tr, locale, "elections-announcement"));
        };
        announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "role", role: role_id.clone()));
    }
    for (operation, users) in operations {
        match operation {
            Operations::Assign => {
                announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "assigned"));
            },
            Operations::Remove => {
                announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "removed"));
            },
        }
        for user in users {
            announcement += &user.mention().to_string();
        }
    }
    let Some(scheduled_for) = scheduled_for else { return format!("{}\n", announcement) };
    if !announcement.is_empty() {
        announcement += &crate::loc!(tr, locale, "elections-announcement", "scheduled_for", date: scheduled_for.to_string(), role: role_id)
    };
    format!("{}\n", announcement)
}

pub async fn affected(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    if message.author.bot { return };
    let Some(db) = &data.db else { return };
    let (Ok(elections), Some(guild)) = (db::entities::votes::Entity::find()
        .inner_join(db::entities::members::Entity)
        .inner_join(db::entities::candidates::Entity)
        .filter(db::entities::members::Column::User.eq(message.author.id.to_string()))
        .group_by(db::entities::candidates::Column::Election)
        .select_only()
        .column(db::entities::candidates::Column::Election)
        .into_tuple::<i64>()
        .all(db)
        .await,
        message.guild(ctx.cache().unwrap())) else { return };

    let mut announcement: String = String::new();

    for election_id in elections {
        let Ok(Some(election)) = db::entities::elections::Entity::find_by_id(election_id).one(db).await else { continue };
        let Ok((operations, scheduled_for))
            = host(ctx, db, &guild, &election, false).await else { continue };
        announcement += &compile_announcement(
            &data.translations, Some(&locale(db, &guild.id.to_string()).await),
            operations, scheduled_for, election.role
        );
    }

    if !announcement.is_empty() {
        let announce_in = if let Ok(Some(guild_db)) = db::entities::guilds::Entity::find()
            .filter(db::entities::guilds::Column::Guild.eq(guild.id.to_string()))
            .one(db)
            .await {
                guild_db.elections_channel.unwrap_or_default()
        }
        else { 
            guild.system_channel_id.and_then(|value| { Some(value.to_string()) }).unwrap_or_default()
        };
        let announce_in = ChannelId::from(announce_in.parse::<u64>().unwrap());
        let _ = announce_in.send_message(ctx.http(), move |r: &mut CreateMessage<'_>| -> &mut CreateMessage<'_> {
            r
            .content(announcement)
        }).await;
    }
}


