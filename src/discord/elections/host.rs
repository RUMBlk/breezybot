use std::collections::HashMap;
use std::collections::HashSet;

use chrono::NaiveDate;
use poise::serenity_prelude::Member;
use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::{ User, CacheHttp, Message, Role, RoleId, Guild, CreateMessage, ChannelId, prelude::SerenityError };
use chrono;
use sea_orm::DatabaseConnection;
use std::hash::Hash;
use num_traits::cast::ToPrimitive;

use crate::localization::Translations;
use crate::Data;
use crate::database as db;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Operations {
    Assign,
    Remove,
}

impl Operations {
    pub async fn apply<'a>(
        &self,
        cache: impl CacheHttp + std::convert::AsRef<poise::serenity_prelude::Http>,
        member: &'a Member,
        role_id: &RoleId
) -> Result<(), SerenityError> {
        match &self {
            Operations::Assign => { 
                member.add_role(cache, role_id).await
            },
            Operations::Remove => {
                member.remove_role(cache, role_id).await
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

    let Ok(candidates) = db::queries::votes::sum_votes(db, election.role.to_u64().unwrap()).await else { return operations };

    let candidates: HashSet<u64> = candidates.into_iter().filter_map(|s| s.parse::<u64>().ok()).collect();
    let role_holders: HashSet<u64> = guild.members
    .into_iter()
    .filter(|(_, member)| member.roles.contains(&role.id))
    .map(|(user_id, _)| Into::<u64>::into(user_id))
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

#[derive(Eq)]
pub struct HostResultUser {
    user: User,
    success: bool,
}

impl HostResultUser {
    pub fn new(user: User, success: bool) -> Self {
        Self { user, success }
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn success(&self) -> bool {
        self.success
    }
}

impl PartialEq for HostResultUser {
    fn eq(&self, other: &Self) -> bool {
        self.user.id == other.user.id
    }
} 

impl Hash for HostResultUser {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user.id.hash(state);
    }
}

pub struct HostResult {
    operations: HashMap<Operations, HashSet<HostResultUser>>,
    next: Option<NaiveDate>,
}

impl HostResult {
    pub fn new(
        operations: HashMap<Operations, HashSet<HostResultUser>>,
        next: Option<NaiveDate>,
    ) -> Self {
        Self { operations, next }
    }

    pub fn operations(&self) -> &HashMap<Operations, HashSet<HostResultUser>> {
        &self.operations
    }

    pub fn next(&self) -> &Option<NaiveDate> {
        &self.next
    }
}

pub enum HostError {
    NotScheduledOrForced,
    ScheduledForLater(NaiveDate),
    RoleUnavailable,
}

pub async fn host(
    cache: &(impl CacheHttp + std::convert::AsRef<poise::serenity_prelude::Http>),
    db: &DatabaseConnection,
    guild: &Guild,
    election: &db::entities::elections::Model,
    force: bool,
) -> Result<HostResult, HostError> {
    let now = chrono::Local::now().date_naive();
    if !force {
        let Some(next) = election.scheduled_date else { return Err(HostError::NotScheduledOrForced) };
        if !(now >= next) { return Err(HostError::ScheduledForLater(next))};
    }
    let election_role = election.role.to_u64().unwrap();
    let role_id = RoleId::from(election_role);
    let Some(role) = guild.roles.get(&role_id) else { 
        let _ = db::queries::elections::delete(db, role_id.get());
        return Err(HostError::RoleUnavailable)
    };


    let mut result = HashMap::new();
    let operations = form_results(db, election, guild.to_owned(), role).await;
    for (operation, user_ids) in operations {
        for user_id in user_ids.to_owned() {
            let Ok(mut member) = guild.member(cache, user_id).await 
            else { 
                let _ = db::queries::candidates::unregister(db, election_role, user_id).await;
                continue
            };

            let success = operation.apply(cache, &mut member, &role_id).await.is_ok();
            result.entry(operation.clone()).or_insert(HashSet::new()).insert(HostResultUser::new((*member).user.clone(), success));
        }
    }

    let next = if let Ok(model) = db::queries::elections::model_schedule_next(db, election.clone()).await {
        model.scheduled_date
    } else {
        election.scheduled_date
    };

    Ok(HostResult::new(result, next))
}

pub fn compile_announcement<'a>(
    announcement: &'a mut String,
    tr: &Translations,
    locale: Option<&'a str>,
    host_result: HostResult,
    role_id: String,
) {
    if host_result.operations().len() > 0 {
        if announcement.is_empty() { 
            *announcement = format!("{}\n", crate::loc!(tr, locale, "elections-announcement"));
        };
        *announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "role", role: role_id.clone()));
    }
    for (operation, users) in host_result.operations() {
        match operation {
            Operations::Assign => {
                *announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "assigned"));
            },
            Operations::Remove => {
                *announcement += &format!("{}\n", crate::loc!(tr, locale, "elections-announcement", "removed"));
            },
        }
        for user in users {
            *announcement += &user.user().mention().to_string();
            if !user.success() { *announcement += &crate::loc!(tr, locale, "elections-announcement", "failed"); } 
        }
    }

    if let Some(next) = host_result.next() {
        if !announcement.is_empty() {
            *announcement += &crate::loc!(
                tr,
                locale,
                "elections-announcement",
                "next",
                date: next.to_string(),
                role: role_id
            );
        }
    }

    *announcement += "\n";
}

pub enum AnnounceError {
    SerenityError(SerenityError),
    ChannelNotSet,
    EmptyAnnouncement,
}

pub async fn announce(
    db: &DatabaseConnection,
    cache: &(impl CacheHttp + std::convert::AsRef<poise::serenity_prelude::Http>),
    guild: &Guild, announcement: String
) -> Result<Message, AnnounceError> {
    if !announcement.is_empty() {
        let announce_in = db::queries::guilds::elections_channel(db, guild.id.get()).await
            .map(|v| Some(ChannelId::from(v.to_u64().unwrap())))
            .unwrap_or(guild.system_channel_id);
        if let Some(channel) = announce_in {
            channel.send_message(cache, CreateMessage::new().content(announcement)).await
                .map_err(|e| AnnounceError::SerenityError(e))
        } else { Err(AnnounceError::ChannelNotSet) }
    } else {
        Err(AnnounceError::EmptyAnnouncement)
    }
}

pub async fn affected(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    if message.author.bot { return };
    let Some(db) = &data.db else { return };
    let Some(guild) = message.guild(&ctx.cache).map(|v| (*v).clone()) else { return };
    let Ok(elections) = db::queries::votes::affected_elections_by_user(db, guild.id.get(), message.author.id.get()).await else { return };

    let mut announcement: String = String::new();

    for id in elections {
        let Some(election) = db::queries::elections::get(db, id).await.expect("") else { continue };
        let Ok(host_result)
            = host(ctx, db, &guild, &election, false).await else { continue };
        compile_announcement(
            &mut announcement,
            &data.translations, Some(&db::queries::guilds::locale(db, guild.id.get()).await),
            host_result, election.role.to_string()
        );
    }

    let _ = announce(db, ctx, &guild, announcement);
}


