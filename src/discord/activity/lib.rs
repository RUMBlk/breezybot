use std::collections::HashMap;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{GuildId, UserId};
use comfy_table::Table;
use sea_orm::DatabaseConnection;

use crate::Context;

use sea_orm;
use sea_orm::{ QueryOrder, QuerySelect };
use sea_orm::{ EntityTrait, QueryFilter, ColumnTrait};
use crate::database as db;

pub async fn stat(
    ctx: Context<'_>,
    locale: String,
) -> String {
    let mut activities = HashMap::<String, i16>::new();

    for presence in ctx.guild().unwrap().presences.values() {
        if let Some(member) = ctx.cache().member::<GuildId, UserId>(ctx.guild_id().unwrap(), presence.user.id) {
            if !member.user.bot {
                for activity in &presence.activities {
                    if activity.name != "Custom Status" {
                        let counter = activities.entry(activity.name.to_string()).or_insert(0);
                        *counter += 1;
                    }
                }
            }
        }
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_header(vec![
            t!("activity.stat.table.activities", locale=&locale),
            t!("activity.stat.table.amount", locale=&locale),
        ]);

    for (activity, amount) in activities {
        table.add_row(vec![activity, amount.to_string()]);
    }

    t!("activity.stat.success", locale=&locale, guild=ctx.guild().unwrap().name, table=table).to_string()
}

pub async fn leaderboard(
    ctx: Context<'_>,
    db: &DatabaseConnection,
    locale: String,
    limit: Option<u64>,
    display_names: Option<bool>,
) -> String {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let response = match db::entities::prelude::Members::find()
    .filter(db::entities::members::Column::Guild.eq(&guild_id))
    .order_by_desc(db::entities::members::Column::Points)
    .limit(limit.unwrap_or(10))
    .all(db)
    .await {
        Ok(db_members) => {
            let mut leaderboard = Table::new();
            leaderboard
            .load_preset(UTF8_FULL_CONDENSED)
            .set_header(vec![
                t!("activity.leaderboard.table.index", locale=&locale),
                t!("activity.leaderboard.table.members", locale=&locale),
                t!("activity.leaderboard.table.points", locale=&locale),
            ]);

            let mut index = 0;
            for db_member in db_members {
                if let Some(member) = ctx.cache().member::<GuildId, u64>(ctx.guild_id().unwrap(), db_member.user.parse().unwrap_or_default()) {
                    index += 1;
                    let username = match display_names.unwrap_or(false) {
                        true => member.display_name().to_string(),
                        false => member.user.name,
                    };
                    leaderboard.add_row(vec![
                        (index).to_string(),
                        username,
                        (db_member.points).to_string(),
                    ]);
                };
            };
            if leaderboard.row_count() > 0 {
                t!("activity.leaderboard.success", locale=&locale, guild=ctx.guild().unwrap().name, table=leaderboard, server_value=db::queries::members::server_value(db, &guild_id).await )
            } else {
                t!("activity.leaderboard.empty", locale=&locale)
            }
        }
        Err(_) => t!("errors.database.unreachable", locale=&locale)
    }
    .to_string();
    response
}