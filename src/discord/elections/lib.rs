use std::ops::Deref;

use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;

use poise::serenity_prelude::{ ChannelId, CreateMessage, GuildId, Role };
use chrono;
use sea_orm::{ ActiveModelTrait, IntoActiveModel, QuerySelect };
use sea_orm::{ DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter };

use crate::Context;
use crate::database as db;
use super::host;

pub async fn leaderboard(
    ctx: Context<'_>,
    db: &DatabaseConnection,
    locale: String,
    role: &Role,
    limit: u64,
    display_names: bool,
) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await {
        Ok(model) => {
            match model {
                Some(election) => {
                    match (
                        db::queries::votes::sum_votes_for_election(election.id)
                        .select_only()
                        .column(db::entities::candidates::Column::User)
                        .column_as(db::entities::members::Column::Points.sum(), "points")
                        .limit(limit)
                        .into_tuple::<(String, sea_orm::prelude::Decimal)>()
                        .all(db)
                        .await,

                        db::entities::votes::Entity::find()
                        .inner_join(db::entities::members::Entity)
                        .inner_join(db::entities::candidates::Entity)
                        .filter(db::entities::candidates::Column::Election.eq(election.id))
                        .select_only()
                        .column_as(db::entities::members::Column::Points.sum(), "total")
                        .into_tuple::<sea_orm::prelude::Decimal>()
                        .one(db)
                        .await
                    ) {
                        (Ok(candidates), Ok(Some(total))) => { 
                            let total: f64 = total.round().try_into().unwrap();
                            let mut table = Table::new();
                            table
                            .load_preset(UTF8_FULL_CONDENSED)
                            .set_header(vec![
                                t!("elections.leaderboard.table.index", locale=&locale),
                                t!("elections.leaderboard.table.candidates", locale=&locale),
                                t!("elections.leaderboard.table.share", locale=&locale),
                            ]);

                            let mut index = 0;
                            for (candidate_id, points) in candidates {
                                if let Some(member) = ctx.cache().member::<GuildId, u64>(ctx.guild_id().unwrap(), candidate_id.parse().unwrap_or_default()) {
                                    index += 1;
                                    let name = match display_names {
                                        true => member.display_name().to_string(),
                                        false => member.user.name,
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
                                t!("elections.leaderboard.success", locale=&locale, role=role.name, table=table)
                            } else {
                                t!("elections.leaderboard.empty", locale=&locale)
                            }
                        },
                        _ => t!("errors.database.oops", locale=&locale),
                    }
                },
                None => t!("elections.leaderboard.election_not_found", locale=&locale, role=role.name),
            }
        },
        Err(_) => dberr,
    }
    .to_string()
}

pub async fn announcements(db: &DatabaseConnection, locale: String, guild_id: &String, channel_id: Option<String>) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    let guild = db::queries::guilds::inselect(db, guild_id).await;
    match guild {
        Some(guild) => {
            let mut guild = guild.into_active_model();
            guild.elections_channel = sea_orm::Set(channel_id.clone());
            match guild.update(db).await {
                Ok(_) => {
                    match channel_id {
                        Some(channel_id) => {
                            t!("elections.announcement.channel", locale=&locale, channel=channel_id)
                        },
                        None => t!("elections.announcement.system", locale=&locale)
                    }
                },
                Err(_) => dberr,
            }
        },
        None => dberr,
    }
    .to_string()
}

pub async fn force(
    ctx: Context<'_>,
    db: &DatabaseConnection,
    locale: String,
    role: &Role,
    ephemeral: &bool,
    mut announce: &bool,
) -> String {
    let dberr = t!("errors.database.oops", locale=&locale).to_string();
    let response = match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await
    {
        Ok(model) => {
            match model {
                Some(election) => {
                    let mut lack_permissions = false;
                    let guild = ctx.guild().unwrap();
                    let operations = host::form_results(db, election.clone(), guild.to_owned(), role).await;
                    let mut announcement = String::new();
                    if operations.len() > 0 {
                        announcement += &format!(
                            "{}\n{}\n",
                            t!("elections.announcement.header", locale=&locale),
                            t!("elections.announcement.role", locale=&locale, role=role.id.to_string()),
                        )
                        .to_string();
                    }
                    for (operation, user_ids) in operations {
                        for user_id in user_ids.to_owned() {
                            let member = guild.member(ctx.http(), user_id).await;
                            match member {
                                Ok(mut member) => {
                                    if let Err(_) = match operation {
                                        host::Operations::Assign => { 
                                            if user_ids.len() > 0 {
                                                announcement += t!("elections.announcement.promoted", locale=&locale).deref();
                                            }
                                            member.add_role(ctx.http(), role.id).await
                                        },
                                        host::Operations::Remove => {
                                            if user_ids.len() > 0 {
                                                announcement += t!("elections.announcement.demoted", locale=&locale).deref();
                                            }
                                            member.remove_role(ctx.http(), role.id).await
                                        }, 
                                    } { lack_permissions = true; };
                                    announcement += &format!("<@{}> ", user_id);
                                },
                                Err(_) => {
                                    host::purge_candidate(db, user_id).await;
                                }
                            }
                            if lack_permissions { announce=&false; break }
                        }
                        announcement += "\n";
                    }
                    let now = chrono::Local::now().date_naive();
                    if let Some(date) = host::schedule_next(db, election, now).await {
                        announcement += t!("elections.announcement.scheduled_for", locale=&locale, date=date.format("%B %-d, %C%y").to_string()).deref();
                    }

                    if lack_permissions { announcement = t!("elections.force.lack_permissions", locale=&locale, role=role.name).to_string(); }

                    if !announcement.is_empty() && *announce {
                        let mut announce_in = match db::entities::guilds::Entity::find()
                        .filter(db::entities::guilds::Column::Guild.eq(guild.id.to_string()))
                        .one(db)
                        .await {
                            Ok(Some(guild_db)) => guild_db.elections_channel.unwrap_or_default(), 
                            _ => String::from(""),
                        };
                        if announce_in.is_empty() { announce_in = guild.system_channel_id.unwrap_or(ctx.channel_id()).to_string() }
                        let announce_in = ChannelId::from(announce_in.parse::<u64>().unwrap());
                        let content = announcement.clone();
                        let _ = announce_in.send_message(ctx.http(), move |r: &mut CreateMessage<'_>| -> &mut CreateMessage<'_> {
                            r
                            .content(content)
                        }).await;
                    }

                    match *ephemeral && !announce && !announcement.is_empty() {
                        true => announcement,
                        false => t!("elections.force.success", locale=&locale, role=role.name).to_string(),
                    }
                },
                None => t!("elections.force.election_not_found", locale=&locale, role=role.name).to_string(),
            }
        },
        Err(_) => dberr,
    };
    response
}
