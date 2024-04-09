mod lib;

use poise::serenity_prelude::{Message, Trigger, CacheHttp, Reaction};
use sea_orm::IntoActiveModel;

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
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            lib::stat(ctx, locale).await
        },
        None => t!("errors.database.unreachable").to_string(),
    };
    let _ = ctx.reply(response).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let response = match &ctx.data().db {
        Some(db) => {
            let locale = db::queries::guilds::locale(db, &ctx.guild_id().unwrap().to_string()).await;
            lib::leaderboard(ctx, db, locale, limit, display_names).await
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
    if !message.author.bot {
        match &data.db {
            Some(db) => {
                if let Some(cache) = ctx.cache() {
                    if let Ok(rules) = message.guild(cache).unwrap().automod_rules(ctx.http()).await {
                        for rule in rules {
                            if let Trigger::Spam = rule.trigger {
                                if rule.exempt_channels.contains(&message.channel_id) { return }
                            }
                        }
                    }    
                }

                if let Some(db_member) = db::queries::members::inselect(db, &message.guild_id.unwrap().to_string(), &message.author.id.to_string()).await {
                    let mut db_member = db_member.into_active_model();
                    let mut reward = 0;
                    reward += (message.content.len() as f64 * 0.1).ceil() as i64; // Content reward
                    reward += (message.attachments.len() + message.sticker_items.len() + message.embeds.len()) as i64; // Attachment, sticker, and embeds reward
    
                    db_member.points = Set(db_member.points.unwrap() + reward);
                    let _ = db_member.update(db).await;
                }
            }
            None => {},
        };
    }
}

pub async fn on_reaction(
    ctx: &poise::serenity_prelude::prelude::Context,
    data: &Data,
    reaction: Reaction,
) {
    match (
        &data.db,
        reaction.message(ctx.http()).await,
        reaction.user(ctx.http()).await,
     ) {
        (Some(db), Ok(message), Ok(reaction_author)) => {
            if !reaction_author.bot && !message.author.bot && reaction_author != message.author  {
                let guild_id = reaction.guild_id.unwrap().to_string();
                match (
                    db::queries::members::inselect(db, &guild_id, &reaction_author.id.to_string()).await,
                    db::queries::members::inselect(db, &guild_id, &message.author.id.to_string()).await,
                ) {
                    (Some(db_message_author), Some(db_reaction_author)) => {
                        let mut db_message_author = db_message_author.into_active_model();
                        let mut db_reaction_author = db_reaction_author.into_active_model();

                        db_message_author.points = Set(db_message_author.points.unwrap() + 10_i64);
                        db_reaction_author.points = Set(db_reaction_author.points.unwrap() + 1_i64);
        
                        let _ = db_message_author.update(db).await;
                        let _ = db_reaction_author.update(db).await;
                    },
                    _ => {},
                }
            }
        }
        _ => {},
    };
}

/*
    let _idk =reply.edit(*self.ctx, move |r: &mut poise::CreateReply<'_>| -> &mut poise::CreateReply<'_> {
        r
        .content(response)
    }).await;
 */