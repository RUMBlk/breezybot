mod lib;

use poise::serenity_prelude::{Message, Trigger, CacheHttp, Reaction};
use sea_orm::IntoActiveModel;

use super::localization::loc;
use crate::Data;
use crate::Context;
use crate::Error;

use sea_orm;
use sea_orm::{ ActiveModelTrait, Set };
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
    let _ = ctx.reply(lib::stat(ctx).await).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = if let Some(db) = &ctx.data().db {
        lib::leaderboard(ctx, db, limit, display_names).await
    } else { loc!(ctx, "error", "database-unreachable") };
    let _ = ctx.reply(response).await;
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
        message.guild_id.and_then(|value| {Some(value.to_string())}),
    ) else { return };
    
    if let Some(guild) = message.guild(cache)
    {
        if let Ok(rules) = guild.automod_rules(ctx.http()).await {
            for rule in rules {
                if let Trigger::Spam = rule.trigger {
                    if rule.exempt_channels.contains(&message.channel_id) { return }
                }
            }
        }
    }  

    if let Some(db_member) = db::queries::members::inselect(db, &guild_id, &message.author.id.to_string()).await {
        let mut db_member = db_member.into_active_model();
        let mut reward = 0;
        reward += (message.content.len() as f64 * 0.1).ceil() as i64; // Content reward
        reward += (message.attachments.len() + message.sticker_items.len() + message.embeds.len()) as i64; // Attachment, sticker, and embeds reward

        db_member.points = Set(db_member.points.unwrap() + reward);
        let _ = db_member.update(db).await;
    }
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
        reaction.guild_id.and_then(|value| {Some(value.to_string())}),
    )
    else { return };

    if !(reaction_author.bot || message.author.bot) && reaction_author != message.author  {
        if let (Some(db_message_author), Some(db_reaction_author)) = 
        (
            db::queries::members::inselect(db, &guild_id, &reaction_author.id.to_string()).await,
            db::queries::members::inselect(db, &guild_id, &message.author.id.to_string()).await,
        ) {
            let mut db_message_author = db_message_author.into_active_model();
            let mut db_reaction_author = db_reaction_author.into_active_model();

            db_message_author.points = Set(db_message_author.points.unwrap() + 10_i64);
            db_reaction_author.points = Set(db_reaction_author.points.unwrap() + 1_i64);

            let _ = db_message_author.update(db).await;
            let _ = db_reaction_author.update(db).await;
        };
    }
}

/*
    let _idk =reply.edit(*self.ctx, move |r: &mut poise::CreateReply<'_>| -> &mut poise::CreateReply<'_> {
        r
        .content(response)
    }).await;
 */