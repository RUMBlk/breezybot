pub mod host;
pub mod lib;
pub mod claims;
pub mod roles;
pub mod votes;

use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use sea_orm::DbErr;
use poise::serenity_prelude::{ Message, Role, User, UserId, Channel };

use crate::Data;
use crate::Context;
use crate::Error; 
use crate::database as db;
use super::loc;

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
            "claims::claims_add",
            "claims::claims_remove",
            "claims::claims_kick",
            "claims::claims_ban",
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
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().and_then(|v| { Some((*v).clone())}) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found",role: role.name) };
        let (Ok(candidates), Ok(Some(total))) = (
            db::queries::votes::leaderboard(db, role.id.get(), limit.unwrap_or(10)).await,
            db::queries::votes::points_total(db, role.id.get()).await
        ) else { return loc!(ctx, "errors.database.oops"); };

        let total: f64 = total.round().try_into().unwrap();
        let mut table = Table::new();
        table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_header(vec![
            loc!(ctx, "index"),
            loc!(ctx, "candidates"),
            loc!(ctx, "share"),
        ]);

        let mut index = 0;
        for (candidate_id, points) in candidates {
            //if let Some(member) = ctx.cache().member::<GuildId, u64>(ctx.guild_id().unwrap(), candidate_id.parse().unwrap_or_default()) {
            if let Some(member) = guild.members.get(&UserId::from(candidate_id)) {
                index += 1;
                let name = match display_names.unwrap_or(false) {
                    true => member.display_name().to_string(),
                    false => member.user.name.clone(),
                };
                let points: f64 = points.round().try_into().unwrap();
                table.add_row(vec![
                    (index).to_string(),
                    name,
                    format!("{:.2}%", ((points/total as f64) * 100_f64) as f64),
                ]);
            }
        }
        if table.row_count() > 0 {
            loc!(ctx, "elections-leaderboard", role: role.name, table: table.to_string())
        } else {
            loc!(ctx, "elections-leaderboard-empty")
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn announcements(
    ctx: Context<'_>,
    channel: Option<Channel>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().and_then(|v| { Some((*v).clone())}) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };

        let channel_id = channel.and_then(|v| { Some(v.id().get()) });

        if let Err(e) = db::queries::guilds::upsert_elections_channel(db, guild.id.get(), channel_id).await {
            match e == DbErr::RecordNotInserted { 
                true => loc!(ctx, "elections-channel-not-updated"),
                false => loc!(ctx, "database-oops") 
            }
        } else {
            match channel_id {
                Some(v) => loc!(ctx, "elections-channel", channel: v),
                None => loc!(ctx, "elections-channel-system")
            }
        }
        "a".to_string()
    }.await).await;
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

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_unban(
    ctx: Context<'_>,
    role: Role,
    user: User,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let (Some(guild), Some(author_member)) = (
            ctx.guild().and_then(|v| { Some((*v).clone())}),
            ctx.author_member().await
        )
        else { return loc!(ctx, "cmd-not-in-guild") };
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Some(highest_role) = guild.member_highest_role(&author_member) else { return loc!(ctx, "unknown-highest-role") };
        if *highest_role < role || ctx.guild().unwrap().owner_id != ctx.author().id { return loc!(ctx, "insufficient_role_position", role: role.name); }

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        if let Err(e) = db::queries::candidates::unban(db, role.id.get(), user.id.get()).await {
            if let DbErr::RecordNotInserted = e {
                loc!(ctx, "elections-claim-already-banned", role: role.name, user: user.name )
            } else { loc!(ctx, "database-oops") }
        } else {
            loc!(ctx, "elections-claim-banned", role: role.name, user: user.name )
        }
    }.await).await;
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



