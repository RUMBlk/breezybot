pub mod host;
pub mod lib;
pub mod roles;
pub mod claims;
pub mod votes;

use std::ops::Deref;

use poise::serenity_prelude::{ Message, Role, User, Channel };

use crate::Data;
use crate::Context;
use crate::Error; 
use crate::database as db;
use super::lib as dstools;

pub fn commands() -> Vec<poise::Command<Data, Box<dyn std::error::Error + Send + Sync>>> {
    vec![elections()]
}

#[poise::command(slash_command, subcommands(
            "leaderboard",
            "announcements",
            "force",
            "roles_list",
            "roles_add",
            "roles_edit",
            "roles_remove",
            "claims_add",
            "claims_remove",
            "claims_kick",
            "claims_ban",
            "claims_unban",
            "votes_list",
            "votes_add", 
            "votes_remove"
        )
    )
]

pub async fn elections(
    _ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    role: Role,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            lib::leaderboard(ctx, db, locale, &role, limit.unwrap_or(10), display_names.unwrap_or(false)).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn announcements(
    ctx: Context<'_>,
    channel: Option<Channel>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            let channel_id = match channel {
                Some(channel) => Some(channel.id().to_string()),
                None => None,
            };
            lib::announcements(db, locale, &ctx.guild_id().unwrap().to_string(), channel_id).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn force(
    ctx: Context<'_>,
    role: Role,
    ephemeral: Option<bool>,
    announce: Option<bool>,
) -> Result<(), Error> {
    let ephemeral = ephemeral.unwrap_or(false);
    let announce = announce.unwrap_or(true);
    match ephemeral {
        true => { let _ = ctx.defer_ephemeral().await; },
        false => { let _ = ctx.defer().await; },
    }

    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            let (_, role_position) = ctx.author_member().await.unwrap().highest_role_info(ctx.cache()).unwrap();
            match role.position < role_position || ctx.guild().unwrap().owner_id == ctx.author().id {
                true => lib::force(ctx, db, locale, &role, &ephemeral, &announce).await,
                false => t!("errors.insufficient_role_position", locale=&locale).to_string(),
            } 
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}


#[poise::command(slash_command)]
pub async fn roles_list(
    ctx: Context<'_>,
    limit: Option<u8>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            roles::list(ctx, db, locale, limit).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_add(
    ctx: Context<'_>,
    role: Role,
    number_of_positions: Option<i16>,
    schedule: Option<roles::TimePeriods>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            roles::add(ctx, db, locale, role, number_of_positions, schedule).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_edit(
    ctx: Context<'_>,
    role: Role,
    number_of_positions: Option<i16>,
    schedule: Option<roles::TimePeriods>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            roles::edit(db, locale, role, number_of_positions, schedule).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_remove(
    ctx: Context<'_>,
    role: Role,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            roles::delete(db, locale, role).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn claims_add(
    ctx: Context<'_>,
    role: Role,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            claims::add(ctx, db, locale, role, ctx.author().to_owned()).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn claims_remove(
    ctx: Context<'_>,
    role: Role,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            claims::delete(ctx, db, locale, role, ctx.author().to_owned()).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_kick(
    ctx: Context<'_>,
    role: Role,
    user: User,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            let highest_role_position = match dstools::highest_role(ctx, ctx.author_member().await.unwrap().deref()).await {
                Some(role) => role.position,
                None => -1,
            };
            
            match role.position < highest_role_position || ctx.guild().unwrap().owner_id == ctx.author().id {
                true => claims::delete(ctx, db, locale, role, user).await,
                false => t!("errors.discord.insufficient_role_position", locale=&locale, role=role.name).to_string(),
            }
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_ban(
    ctx: Context<'_>,
    role: Role,
    user: User,
    days: Option<u8>,
    weeks: Option<u8>,
    months: Option<u8>,
    years: Option<u8>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            /*let has_permissions = match ctx.author_member().await.unwrap().highest_role_info(ctx.cache()) {
                Some((_, position)) => role.position < position,
                None => ctx.guild().unwrap().owner_id == ctx.author().id,
            };*/

            let highest_role_position = match dstools::highest_role(ctx, ctx.author_member().await.unwrap().deref()).await {
                Some(role) => role.position,
                None => -1,
            };
            
            match role.position < highest_role_position || ctx.guild().unwrap().owner_id == ctx.author().id {
                true => claims::ban(db, locale, role, user, days, weeks, months, years).await,
                false => t!("errors.discord.insufficient_role_position", locale=&locale, role=role.name).to_string(),
            }
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_unban(
    ctx: Context<'_>,
    role: Role,
    user: User,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            let highest_role_position = match dstools::highest_role(ctx, ctx.author_member().await.unwrap().deref()).await {
                Some(role) => role.position,
                None => -1,
            };
            
            match role.position < highest_role_position || ctx.guild().unwrap().owner_id == ctx.author().id {
                true => claims::unban(db, locale, role, user).await,
                false => t!("errors.discord.insufficient_role_position", locale=&locale, role=role.name).to_string(),
            }
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn votes_list(
    ctx: Context<'_>,
    role: Role,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            votes::list(ctx, db, locale, ctx.guild_id().unwrap().to_string(), ctx.author().id.to_string(), role, display_names).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn votes_add(
    ctx: Context<'_>,
    role: Role,
    candidate: User,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            votes::add(db, locale, ctx.guild_id().unwrap().to_string(), ctx.author().id.to_string(),role, candidate).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn votes_remove(
    ctx: Context<'_>,
    role: Role,
    candidate: User,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            votes::delete(db, locale, ctx.guild_id().unwrap().to_string(), ctx.author().id.to_string(),role, candidate).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

pub async fn on_message(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    host::affected(ctx, data, message).await;
}



