pub mod host;
pub mod claims;
pub mod roles;
pub mod votes;

use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use sea_orm::DbErr;
use poise::serenity_prelude::{ Message, Role, UserId, Channel };

use crate::Data;
use crate::Context;
use crate::Error; 
use crate::database as db;
use super::{ loc, lib };

pub fn commands() -> Vec<poise::Command<Data, Box<dyn std::error::Error + Send + Sync>>> {
    vec![elections()]
}

#[poise::command(slash_command, subcommands(
            "leaderboard",
            "announcements",
            "force",
            "roles::roles_list",
            "roles::roles_add",
            "roles::roles_edit",
            "roles::roles_remove",
            "claims::claims_add",
            "claims::claims_remove",
            "claims::claims_kick",
            "claims::claims_ban",
            "claims::claims_unban",
            "votes::votes_list",
            "votes::votes_cast", 
            "votes::votes_remove"
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
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found",role: role.name) };
        let (Ok(candidates), Some(total)) = (
            db::queries::votes::leaderboard(db, role.id.get(), limit.unwrap_or(10)).await,
            db::queries::votes::points_total(db, role.id.get()).await
        ) else { return loc!(ctx, "database-oops"); };

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
            } else {
                let _ = db::queries::candidates::unregister(db, role.id.get(), candidate_id);
            }
        }
        if table.row_count() > 0 {
            loc!(ctx, "elections-leaderboard-table", role: role.name, table: table.to_string())
        } else {
            loc!(ctx, "elections-leaderboard-table-empty")
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
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };

        let channel_id = channel.clone().and_then(|v| { Some(v.id().get()) });


        if let Err(e) = db::queries::guilds::upsert_elections_channel(db, guild.id.get(), channel_id).await {
            match e == DbErr::RecordNotInserted { 
                true => loc!(ctx, "elections-channel-not-updated"),
                false => loc!(ctx, "database-oops") 
            }
        } else {
            match channel {
                Some(v) => loc!(ctx, "elections-channel", channel: v.to_string() ),
                None => loc!(ctx, "elections-channel-system")
            }
        }
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

    let _ = ctx.reply(async move {
        let (Some(guild), Some(author_member)) = (
            ctx.guild().map(|v| (*v).clone()),
            ctx.author_member().await
        ) else { return loc!(ctx, "cmd-not-in-guild") };
        let Ok(hsr) = lib::has_sufficient_role(&guild, &author_member, &role).await else { return loc!(ctx, "unknown-highest-role") };
        if !hsr { return loc!(ctx, "insufficient_role_position", role: &role.name) }
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Some(election) = db::queries::elections::get(db, role.id.get()).await.expect("")
            else { return loc!(ctx, "election-not-found"); };
    
        let mut announcement = String::new();
    
        match host::host(&ctx, db, &guild, &election, true).await.map_err(|e| {
            match e {
                host::HostError::NotScheduledOrForced => loc!(ctx, "elections-not-scheduled", role: &role.name),
                host::HostError::RoleUnavailable => loc!(ctx, "role-unavaiable"),
                host::HostError::ScheduledForLater(scheduled_date) => loc!(ctx, "elections-scheduled-for-later", role: &role.name, scheduled_date: scheduled_date.to_string()),
            }
        }) {
            Ok(host_result) => {
                host::compile_announcement(
                    &mut announcement,
                    &ctx.data().translations, Some(&db::queries::guilds::locale(db, guild.id.get()).await),
                    host_result, election.role.to_string());
            },
            Err(e) => return e,
        };
    
        if announce {
            let _ = host::announce(db, &ctx, &guild, announcement.clone()).await;
        }
    
        match ephemeral && !announce && !announcement.is_empty() {
            true => announcement,
            false => loc!(ctx, "elections-forced", role: &role.name),
        }
    }.await).await;
    Ok(())
}

pub async fn on_message(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    host::affected(ctx, data, message).await;
}



