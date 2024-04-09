use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};
use sea_orm::{ EntityTrait, QueryFilter, ColumnTrait};
use poise::serenity_prelude::{ Mentionable, Role, User };
use crate::database as db;
use crate::Context;

pub async fn add(ctx: Context<'_>, db: &DatabaseConnection, locale: String, role: Role, user: User) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await {
        Ok(model) => { 
            match model {
                Some(row) => {
                    match db::entities::candidates::Entity::insert(
                        db::entities::candidates::ActiveModel {
                            election: sea_orm::Set(row.id),
                            user: sea_orm::Set(user.id.to_string()),
                            ..Default::default()
                        }
                    )
                    .exec(db)
                    .await {
                        Ok(_) => {
                            let username = match user.nick_in(ctx, ctx.guild_id().unwrap()).await {
                                Some(name) => name,
                                None => user.name,
                            };
                            t!("elections.claims.add.success", locale=&locale, role=role.name, user=username)
                        },
                        Err(_) => t!("elections.claims.add.exists", locale=&locale, role=role.name),
                    }
                },
                None => t!("elections.claims.add.elections_not_found", locale=&locale, role=role.name),
            }
        },
        Err(_) => dberr,
    }
    .to_string()
}

pub async fn delete(ctx: Context<'_>, db: &DatabaseConnection, locale: String, role: Role, user: User) -> String {
    let dberr = t!("errors.database.oops", locale=&locale);
    match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await {
        Ok(model) => {
            match model {
                Some(election) => {
                    match db::entities::candidates::Entity::find()
                    .filter(
                        sea_orm::Condition::all()
                        .add(db::entities::candidates::Column::Election.eq(election.id))
                        .add(db::entities::candidates::Column::User.eq(user.id.to_string()))
                    )
                    .one(db)
                    .await {
                        Ok(claim_model) => {
                            match claim_model {
                                Some(claim) => {
                                    match claim.banned_until.unwrap_or_default() > chrono::Local::now() {
                                        true => {
                                            match db::entities::candidates::Entity::delete(claim.into_active_model()).exec(db).await {
                                                Ok(_) => { 
                                                    let username = match user.nick_in(ctx, ctx.guild_id().unwrap()).await {
                                                        Some(name) => name,
                                                        None => user.name,
                                                    };
                                                    t!("elections.claims.delete.success", locale=&locale, role=role.name, user=username)
                                                },
                                                Err(_) => dberr,
                                            }
                                        },
                                        false => {
                                            t!("elections.claims.delete.banned", locale=&locale, role=role.name)
                                        }
                                    }
                                }
                                None => t!("elections.claims.delete.claim_not_found", locale=&locale, role=role.name),
                            }
                        }
                        Err(_) => dberr,
                    }
                },
                None => t!("elections.claims.delete.election_not_found", locale=&locale, role=role.name),
            } 
        },
        Err(_) => dberr,
    }
    .to_string()
}

pub async fn edit_ban(db: &DatabaseConnection, locale: &String, role: &Role, user: &User, banned_until: Option<chrono::DateTime<chrono::FixedOffset>>) -> Option<String> {
    let dberr = Some(t!("errors.database.oops", locale=&locale).to_string());
    match db::entities::elections::Entity::find()
    .filter(db::entities::elections::Column::Role.eq(role.id.to_string()))
    .one(db)
    .await {
        Ok(model) => {
            match model {
                Some(election) => {
                    let _ = db::entities::candidates::Entity::insert(
                        db::entities::candidates::ActiveModel {
                            election: sea_orm::Set(election.id),
                            user: sea_orm::Set(user.id.to_string()),
                            ..Default::default()
                        }
                    )
                    .exec(db)
                    .await;

                    match db::entities::candidates::Entity::find()
                    .filter(
                        sea_orm::Condition::all()
                        .add(db::entities::candidates::Column::Election.eq(election.id))
                        .add(db::entities::candidates::Column::User.eq(user.id.to_string()))
                    )
                    .one(db)
                    .await {
                        Ok(claim_model) => {
                            match claim_model {
                                Some(claim) => {
                                    let mut claim = claim.into_active_model();
                                    claim.banned_until = sea_orm::Set(banned_until);
                                    match claim.update(db).await {
                                        Ok(_) => None,
                                        Err(_) => dberr,
                                    }
                                }
                                None => Some(t!("elections.claims.ban.claim_not_found", locale=&locale, role=role.name).to_string()),
                            }
                        }
                        Err(_) => dberr,
                    }
                },
                None => Some(t!("elections.claims.ban.election_not_found", locale=&locale, role=role.name).to_string()),
            } 
        },
        Err(_) => dberr,
    }
}

pub async fn ban(db: &DatabaseConnection, locale: String, role: Role, user: User, days: Option<u8>, weeks: Option<u8>, months: Option<u8>, years: Option<u8>) -> String {
    let mut banned_until = chrono::Local::now();
    if let Some(days) = days { banned_until = banned_until.checked_add_days(chrono::Days::new(days.into())).unwrap_or(banned_until);  }
    if let Some(weeks) = weeks { banned_until = banned_until.checked_add_days(chrono::Days::new(7_u64*weeks as u64)).unwrap_or(banned_until);  }
    if let Some(months) = months { banned_until = banned_until.checked_add_months(chrono::Months::new(months.into())).unwrap_or(banned_until); }
    if let Some(years) = years { banned_until = banned_until.checked_add_months(chrono::Months::new(12_u32*years as u32)).unwrap_or(banned_until); }
    match edit_ban(db, &locale, &role, &user, Some(banned_until.into())).await {
        Some(response) => response,
        None => t!("elections.claims.ban.success", locale=&locale, role=role.name, user=user.mention(), banned_until=banned_until.timestamp()).to_string(),
    }
}

pub async fn unban(db: &DatabaseConnection, locale: String, role: Role, user: User) -> String {
    match edit_ban(db, &locale, &role, &user, None).await {
        Some(response) => response,
        None => t!("elections.claims.unban.success", locale=&locale, role=role.name, user=user.mention()).to_string(),
    }
}