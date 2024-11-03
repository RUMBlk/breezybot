use sea_orm::DbErr;
use poise::serenity_prelude::{ Role, User };

use crate::Context;
use crate::Error; 
use crate::database as db;
use super::{ loc, lib };

#[poise::command(slash_command)]
pub async fn claims_add(
    ctx: Context<'_>,
    role: Role,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let Some(guild_id) = ctx.guild_id() else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };

        if let Ok(Some(is_active)) = db::queries::candidates::is_active(db, role.id.get(), ctx.author().id.get()).await {
            if is_active { return loc!(ctx, "elections-candidate-exists", role: role.name) }
        };
        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        if let Ok(_) = db::queries::candidates::register(db, role.id.get(), ctx.author().id.get()).await {
            loc!(ctx, "elections-claim-added", role: role.name,
                user: ctx.author().nick_in(ctx, guild_id).await.unwrap_or(ctx.author().name.clone())
            )
        } else { loc!(ctx, "database-oops") }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn claims_remove(
    ctx: Context<'_>,
    role: Role,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        if ctx.guild().is_none() { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        let username = ctx.author().nick_in(ctx, ctx.guild_id().unwrap()).await.unwrap_or(ctx.author().name.clone());

        if let Ok(v) = db::queries::candidates::unregister(db, role.id.get(), ctx.author().id.get()).await {
            if v.rows_affected == 0 { loc!(ctx, "elections-claim-not-found", role: role.name) }
            else {  loc!(ctx, "elections-claim-removed", role: role.name, user: username) }
        } else { loc!(ctx, "database-oops") }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_kick(
    ctx: Context<'_>,
    role: Role,
    user: User,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let (Some(guild), Some(author_member)) = (
            ctx.guild().map(|v| (*v).clone()),
            ctx.author_member().await
        )
        else { return loc!(ctx, "cmd-not-in-guild") };
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Ok(hsr) = lib::has_sufficient_role(&guild, &author_member, &role).await else { return loc!(ctx, "unknown-highest-role") };
        if !hsr { return loc!(ctx, "insufficient_role_position", role: role.name) }

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        if let Ok(v) = db::queries::candidates::unregister(db, role.id.get(), user.id.get()).await {
            if v.rows_affected == 0 { loc!(ctx, "elections-candidate-not-found", role: role.name, user: user.name) }
            else {  loc!(ctx, "elections-claim-removed", role: role.name, candidate: user.name ) }
        } else { loc!(ctx, "database-oops") }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="MANAGE_ROLES")]
pub async fn claims_ban(
    ctx: Context<'_>,
    role: Role,
    user: User,
    days: Option<i64>,
    weeks: Option<i64>,
    years: Option<i64>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let (Some(guild), Some(author_member)) = (
            ctx.guild().map(|v| (*v).clone()),
            ctx.author_member().await
        )
        else { return loc!(ctx, "cmd-not-in-guild") };
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Ok(hsr) = lib::has_sufficient_role(&guild, &author_member, &role).await else { return loc!(ctx, "unknown-highest-role") };
        if !hsr { return loc!(ctx, "insufficient_role_position", role: role.name) }

        let duration = chrono::Duration::days(days.unwrap_or_default())
            .checked_add(&chrono::Duration::weeks(weeks.unwrap_or_default()))
            .and_then(|v| { v.checked_add(&chrono::Duration::days(365*years.unwrap_or_default())) })
            .unwrap_or_default();

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        match db::queries::candidates::ban(db, role.id.get(), user.id.get(), duration).await {
            Ok(v) => {
                loc!(
                    ctx,
                    "elections-claim-banned",
                    role: role.name,
                    user: user.name,
                    banned_until: v.banned_until.map(|v| v.timestamp()).unwrap_or(chrono::Utc::now().timestamp())
                )
            },
            Err(e) => {
                if let DbErr::RecordNotInserted = e {
                    loc!(ctx, "elections-claim-already-banned", role: role.name, user: user.name )
                } else { loc!(ctx, "database-oops") }
            }
        }
    }.await).await;
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
            ctx.guild().map(|v| (*v).clone()),
            ctx.author_member().await
        )
        else { return loc!(ctx, "cmd-not-in-guild") };
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Some(highest_role) = guild.member_highest_role(&author_member) else { return loc!(ctx, "unknown-highest-role") };
        if *highest_role < role || ctx.guild().unwrap().owner_id != ctx.author().id { return loc!(ctx, "insufficient_role_position", role: role.name); }

        if !db::queries::elections::exists(db, role.id.get()).await { return loc!(ctx, "elections-not-found", role: role.name) }

        if let Ok(v) = db::queries::candidates::unban(db, role.id.get(), user.id.get()).await {
            if v.rows_affected == 0 {
                loc!(ctx, "elections-claim-not-banned", role: role.name, user: user.name)
            } else {
                loc!(ctx, "elections-claim-unbanned", role: role.name, user: user.name )
            }
        } else {
            loc!(ctx, "database-oops")
        }
    }.await).await;
    Ok(())
}
