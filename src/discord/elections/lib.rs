use std::ops::Deref;

use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;

use poise::serenity_prelude::{ ChannelId, CreateMessage, UserId, Role, Guild };
use chrono;
use sea_orm::{ ActiveModelTrait, IntoActiveModel, QuerySelect };
use sea_orm::{ DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter };

use crate::Context;
use crate::database as db;
use super::host;

pub async fn post_in_the_channel(ctx: Context<'_>, db: &DatabaseConnection, guild: Guild, content: String) {
    let mut announce_in = match db::entities::guilds::Entity::find()
    .filter(db::entities::guilds::Column::Guild.eq(guild.id.to_string()))
    .one(db)
    .await {
        Ok(Some(guild_db)) => guild_db.elections_channel.unwrap_or_default(), 
        _ => String::from(""),
    };
    if announce_in.is_empty() { announce_in = guild.system_channel_id.unwrap_or(ctx.channel_id()).to_string() }
    let announce_in = ChannelId::from(announce_in.parse::<u64>().unwrap());
    let _ = announce_in.send_message(ctx.http(), CreateMessage::new().content(content)).await;
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
                    let operations = host::form_results(db, &election.clone(), guild.to_owned(), role).await;
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
                        match operation {
                            host::Operations::Assign => { 
                                if user_ids.len() > 0 {
                                    announcement += t!("elections.announcement.promoted", locale=&locale).deref();
                                }
                            },
                            host::Operations::Remove => {
                                if user_ids.len() > 0 {
                                    announcement += t!("elections.announcement.demoted", locale=&locale).deref();
                                }
                            }, 
                        } 
                        for user_id in user_ids.to_owned() {
                            let member = guild.member(ctx.http(), user_id).await;
                            match member {
                                Ok(mut member) => {
                                    if let Err(_) = match operation {
                                        host::Operations::Assign => { 
                                            member.add_role(ctx.http(), role.id).await
                                        },
                                        host::Operations::Remove => {
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
                    if let Some(date) = host::schedule_next(db, &election, now).await {
                        announcement += t!("elections.announcement.scheduled_for", locale=&locale, role=role.name, date=date.format("%B %-d, %C%y").to_string()).deref();
                    }

                    if lack_permissions { announcement = t!("elections.force.lack_permissions", locale=&locale, role=role.name).to_string(); }

                    if !announcement.is_empty() && *announce {
                        post_in_the_channel(ctx, db, guild, announcement.clone()).await;
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
