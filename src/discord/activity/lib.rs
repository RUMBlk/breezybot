use std::collections::HashMap;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{GuildId, UserId};
use comfy_table::Table;
use sea_orm::DatabaseConnection;

use super::loc;
use crate::Context;

use crate::database as db;

pub async fn stat(ctx: Context<'_>) -> String {
    let (Some(guild), Some(guild_id)) 
        = (ctx.guild(), ctx.guild_id()) else { return loc!(ctx, "cmd-not-in-guild") }; 
    let mut activities = HashMap::<String, i16>::new();

    for presence in guild.presences.values() {
        let Some(member) = ctx.cache().member::<GuildId, UserId>(guild_id, presence.user.id) else { continue };
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

    loc!(ctx, "activity-stat-table", guild: guild.name, table: table.to_string())
}

pub async fn leaderboard(
    ctx: Context<'_>,
    db: &DatabaseConnection,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> String {
    let (Some(guild), Some(guild_id)) 
        = (ctx.guild(), ctx.guild_id()) else { return loc!(ctx, "cmd-not-in-guild") }; 

    let Ok(db_members) = db::queries::members::leaderboard(&guild_id.to_string(), limit)
        .all(db)
        .await else { return loc!(ctx, "database-unreachable") };

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
        let Ok(userid) = db_member.user.parse()
            .map_err(|value| { eprintln!("{}", value); }) 
        else { return loc!(ctx, "database-oops") };
        
        if let Some(member) = ctx.cache().member::<GuildId, u64>(guild_id, userid) {
            index += 1;
            username = match display_names.unwrap_or(false) {
                true => member.display_name().to_string(),
                false => member.user.name,
            };
        } else {
            username = db_member.user;
        };

        leaderboard.add_row(vec![
            (index).to_string(),
            username,
            (db_member.points).to_string(),
        ]);
    };

    let server_value = db::queries::members::server_value(db, &guild_id.to_string()).await
        .ok()
        .unwrap_or_default()
        .and_then(|v| { Some(v.floor().to_string()) })
        .unwrap_or(loc!(ctx, "activity-leaderboard-table", "server-value-err"));

    if leaderboard.row_count() > 0 {
        loc!(
                ctx, "activity-leaderboard-table",
                guild: guild.name, table: leaderboard.to_string(),
                server_value: server_value,
        )
    } else {
        loc!(ctx, "activity-leaderboard-empty")
    }
}