use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{ Mentionable, Role, User, UserId };
use num_traits::cast::ToPrimitive;

use crate::Context;
use crate::Error; 
use crate::database as db;
use super::loc;

#[poise::command(slash_command)]
pub async fn votes_list(
    ctx: Context<'_>,
    role: Role,
    display_names: Option<bool>,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
    
        let Ok(candidates) = db::queries::candidates::list_by_voter(db, role.id.get(), ctx.author().id.get()).await
            else { return loc!(ctx, "errors.database.oops"); };
    
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_header(vec![
                loc!(ctx, "index"),
                loc!(ctx, "candidates"),
            ]);
    
        let mut index = 0;
        for (_, user_id) in candidates {
            let user_id = user_id.to_u64().unwrap();
            if let Some(member) = guild.members.get(&UserId::from(user_id)) {
                index += 1;
                let username = match display_names.unwrap_or(false) {
                    true => member.display_name().to_string(),
                    false => member.user.name.clone(),
                };
                table.add_row(vec![index.to_string(), username]);
            } else {
                let _ = db::queries::candidates::unregister(db, role.id.get(), user_id).await;
            }
        };
        if index > 0 {
            loc!(ctx, "elections-votes-list", role: role.name, table: table.to_string())
        } else {
            loc!(ctx, "elections-votes-list-empty", role: role.name)
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn votes_cast(
    ctx: Context<'_>,
    role: Role,
    candidate: User,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
    
        let Ok(member_db) = db::queries::members::upsert(db, guild.id.get(), ctx.author().id.get()).await else { return loc!(ctx, "database-oops"); };
    
        let candidate_mention = candidate.mention().to_string();
        let Some(candidate_id) = db::queries::candidates::id(db, role.id.get(), candidate.id.get()).await.expect("")
            else { return loc!(ctx, "elections-candidate-not-found", role: role.name, candidate: candidate.mention().to_string()); };
        
        if let Ok(_) = db::queries::votes::cast(db, member_db.id, candidate_id).await {
            loc!(ctx, "elections-vote-casted", role: role.mention().to_string(), candidate: candidate_mention)
        } else {
            loc!(ctx, "elections-vote-exists", role: role.mention().to_string(), candidate: candidate_mention)
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn votes_remove(
    ctx: Context<'_>,
    role: Role,
    candidate: User,
) -> Result<(), Error> {
    let _ = ctx.defer_ephemeral().await;
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild") }; 
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
    
        let Ok(member_db) = db::queries::members::upsert(db, guild.id.get(), ctx.author().id.get()).await else { return loc!(ctx, "database-oops"); };
    
        let candidate_mention = candidate.mention().to_string();
        let Some(candidate_id) = db::queries::candidates::id(db, role.id.get(), candidate.id.get()).await.expect("")
            else { return loc!(ctx, "elections-candidate-not-found", role: role.name, candidate: candidate.mention().to_string()); };
        
        if let Ok(v) = db::queries::votes::remove(db, member_db.id, candidate_id).await {
            if v.rows_affected != 0 {
                loc!(ctx, "elections-vote-remove", role: role.mention().to_string(), candidate: candidate_mention)
            } else {
                loc!(ctx, "elections-vote-not-found", role: role.mention().to_string(), candidate: candidate_mention)
            }
        } else {
            loc!(ctx, "database-oops")
        }
    }.await).await;
    Ok(())
}
