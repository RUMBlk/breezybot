use poise::serenity_prelude::{Message, Trigger, CacheHttp, Reaction};
use std::{ ops::Deref, collections::HashMap };
use comfy_table::{ Table, presets::UTF8_FULL_CONDENSED };
use num_traits::cast::ToPrimitive;

use super::localization::loc;
use crate::Data;
use crate::Context;
use crate::Error;
use crate::database as db;

pub fn commands() -> Vec<poise::Command<Data, Box<dyn std::error::Error + Send + Sync>>> {
    vec![activity()]
}

#[poise::command(slash_command, subcommands("stat", "leaderboard"))]
pub async fn activity(
    _ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stat(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let Some(guild)
            = ctx.guild().and_then(|v| {Some(v.deref().clone())})
        else { return loc!(ctx, "cmd-not-in-guild") }; 
        let mut activities = HashMap::<String, i16>::new();

        for presence in guild.presences.values() {
            let Ok(member) = guild.member(ctx, presence.user.id).await else { continue };
            if member.user.bot { continue };
            for activity in &presence.activities {
                if activity.name != "Custom Status" {
                    let counter = activities.entry(activity.name.to_string()).or_insert(0);
                    *counter += 1;
                }
            }
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_header(vec![
                loc!(ctx, "activities"),
                loc!(ctx, "participants"),
            ]);

        for (activity, amount) in activities {
            table.add_row(vec![activity, amount.to_string()]);
        }

        if table.row_count() > 0 {
            loc!(ctx, "activity-stat-table", guild: guild.name, table: table.to_string())
        } else {
            loc!(ctx, "activity-stat-empty")
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let _ = ctx.defer();
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        
        let Some(guild)
            = ctx.guild().and_then(|v| {Some(v.deref().clone())}) else { return loc!(ctx, "cmd-not-in-guild") }; 

        let Ok(db_members) 
            = db::queries::members::leaderboard(db, guild.id.get(), limit).await
            else { return loc!(ctx, "database-oops") };

        let mut leaderboard = Table::new();
        leaderboard
        .load_preset(UTF8_FULL_CONDENSED)
        .set_header(vec![
            loc!(ctx, "index"),
            loc!(ctx, "members"),
            loc!(ctx, "points"),
        ]);

        let mut index = 0;
        for db_member in db_members {
            let username;
            
            if let Ok(member) = guild.member(ctx, db_member.user.to_u64().unwrap()).await {
                index += 1;
                username = match display_names.unwrap_or(false) {
                    true => member.display_name().to_string(),
                    false => (*member).user.name.clone(),
                };
            } else {
                username = db_member.user.to_string();
            };

            leaderboard.add_row(vec![
                (index).to_string(),
                username,
                (db_member.points).to_string(),
            ]);
        };

        let server_value = db::queries::members::server_value(db, guild.id.get()).await
            .map(|v| v.floor().to_string())
            .unwrap_or(loc!(ctx, "activity-leaderboard-table", "server-value-err"));

        if leaderboard.row_count() > 0 {
            loc!(
                ctx, "activity-leaderboard-table",
                guild: guild.name.clone(), table: leaderboard.to_string(),
                server_value: server_value,
            )
        } else {
            loc!(ctx, "activity-leaderboard-empty")
        }
    }.await).await;
    Ok(())
}

pub async fn on_message(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    message: Message,
) {
    if message.author.bot { return }
    let (
        Some(db),
        Some(cache),
        Some(guild_id),
    ) = (
        &data.db,
        ctx.cache(),
        message.guild_id,
    ) else { return };
    
    if let Some(guild) = message.guild(cache).and_then(|v| { Some((*v).clone())})
    {
        if let Ok(rules) = guild.automod_rules(ctx.http()).await {
            for rule in rules {
                if let Trigger::Spam = rule.trigger {
                    if rule.exempt_channels.contains(&message.channel_id) { return }
                }
            }
        }
    }  

    let reward = (message.content.len() as f64 * 0.1).ceil() as i64 //Content reward
        + (message.attachments.len() + message.sticker_items.len() + message.embeds.len()) as i64; // Attachment, sticker, and embeds reward
    
    let _ = db::queries::members::add_points(db, guild_id.get(), message.author.id.get(), reward).await;
}

pub async fn on_reaction(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    reaction: Reaction,
) {
    let (
        Some(db),
        Ok(message),
        Ok(reaction_author),
        Some(guild_id),
    ) = (
        &data.db,
        reaction.message(ctx.http()).await,
        reaction.user(ctx.http()).await,
        reaction.guild_id,
    )
    else { return };

    if !(reaction_author.bot || message.author.bot) && reaction_author != message.author  {
        let _ = db::queries::members::add_points(db, guild_id.get(), reaction_author.id.get(), 10).await;
        let _ = db::queries::members::add_points(db, guild_id.get(), message.author.id.get(), 10).await;
    }
}

/*
    let _idk =reply.edit(*self.ctx, move |r: &mut poise::CreateReply<'_>| -> &mut poise::CreateReply<'_> {
        r
        .content(response)
    }).await;
 */