use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{ GuildId, Mentionable, Role, User };
use comfy_table::Table;
use sea_orm::{Condition, DatabaseConnection, IntoActiveModel};
use sea_orm::{ EntityTrait, QueryFilter, ColumnTrait};

use crate::Context;
use crate::database as db;

pub async fn list(ctx: Context<'_>, db: &DatabaseConnection, locale: String, guild_id: String, user_id: String, role: Role, display_names: Option<bool>) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    
    match db::queries::members::inselect(db, &guild_id, &user_id).await {
        Some(member) => {
            match db::entities::candidates::Entity::find()
            .inner_join(db::entities::votes::Entity)
            .inner_join(db::entities::elections::Entity)
            .filter(
                sea_orm::Condition::all()
                .add(db::entities::votes::Column::Member.eq(member.id))
                .add(db::entities::elections::Column::Role.eq(role.id.to_string()))
            )
            .all(db)
            .await {
                Ok(candidates) => {
                    let mut table = Table::new();
                    table
                    .load_preset(UTF8_FULL_CONDENSED)
                    .set_header(vec![
                        t!("elections.votes.list.table.index", locale=&locale),
                        t!("elections.votes.list.table.candidates", locale=&locale),
                    ]);

                    let mut index = 0;
                    for candidate in candidates{
                        if let Some(member) = ctx.cache().member::<GuildId, u64>(ctx.guild_id().unwrap(), candidate.user.parse().unwrap_or_default()) {
                            index += 1;
                            let username = match display_names.unwrap_or(false) {
                                true => member.display_name().to_string(),
                                false => member.user.name,
                            };
                            table.add_row(vec![index.to_string(), username]);
                        }
                    };
                    if index > 0 {
                        t!("elections.votes.list.success", locale=&locale, role=role.name, table=table)
                    } else {
                        t!("elections.votes.list.empty", locale=&locale, role=role.name)
                    }
                },
                Err(_) => dberr,
            }
        }
        None => dberr,
    }
    .to_string()
}

pub async fn add(db: &DatabaseConnection, locale: String, guild_id: String, user_id: String, role: Role, candidate_user: User) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match (
        db::entities::candidates::Entity::find()
        .inner_join(db::entities::elections::Entity)
        .filter(
            Condition::all()
            .add(db::entities::elections::Column::Role.eq(role.id.to_string()))
            .add(db::entities::candidates::Column::User.eq(candidate_user.id.to_string()))
        )
        .one(db)
        .await,

        db::queries::members::inselect(db, &guild_id, &user_id).await,
    ) {
        (Ok(model), Some(member)) => { 
            match model {
                Some(candidate) => {
                    match db::entities::votes::Entity::insert(
                        db::entities::votes::ActiveModel {
                            member: sea_orm::Set(member.id),
                            candidate: sea_orm::Set(candidate.id),
                            ..Default::default()
                        }
                    )
                    .exec(db)
                    .await {
                        Ok(_) => t!("elections.votes.add.success", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
                        Err(_) => t!("elections.votes.add.exists", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
                    }
                },
                None => t!("elections.votes.add.candidate_not_found", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
            }
        },
        _ =>  dberr,
    }
    .to_string()
}

pub async fn delete(db: &DatabaseConnection, locale: String, guild_id: String, user_id: String, role: Role, candidate_user: User) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match (
        db::entities::candidates::Entity::find()
        .inner_join(db::entities::elections::Entity)
        .filter(
            Condition::all()
            .add(db::entities::elections::Column::Role.eq(role.id.to_string()))
            .add(db::entities::candidates::Column::User.eq(candidate_user.id.to_string()))
        )
        .one(db)
        .await,

        db::queries::members::inselect(db, &guild_id, &user_id).await,
    ) {
        (Ok(model), Some(member)) => { 
            match model {
                Some(candidate) => {
                    match db::entities::votes::Entity::find()
                    .filter(
                        sea_orm::Condition::all()
                        .add(db::entities::votes::Column::Member.eq(member.id))
                        .add(db::entities::votes::Column::Candidate.eq(candidate.id))
                    )
                    .one(db)
                    .await {
                        Ok(model) => {
                            match model {
                                Some(vote) => {
                                    match db::entities::votes::Entity::delete(vote.into_active_model()).exec(db).await {
                                        Ok(_) => t!("elections.votes.delete.success", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
                                        Err(_) => dberr,
                                    }
                                },
                                None => t!("elections.votes.delete.vote_not_found", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
                            }
                        },
                        Err(_) => dberr,
                    }
                },
                None => t!("elections.votes.delete.candidate_not_found", locale=&locale, role=role.name, candidate=candidate_user.mention().to_string()),
            }
        },
        _ =>  dberr,
    }
    .to_string()
}