use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use poise::serenity_prelude::{ Role, RoleId };
use num_traits::cast::ToPrimitive;

use crate::Context;
use crate::Error; 
use crate::database as db;
use super::{ loc, lib };

#[derive(poise::ChoiceParameter)]
enum Schedule {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Into<db::queries::elections::Schedule> for Schedule {
    fn into(self) -> db::queries::elections::Schedule {
        use db::queries::elections::Schedule;
        match self {
            Self::Daily => Schedule::Daily,
            Self::Weekly => Schedule::Weekly,
            Self::Monthly => Schedule::Monthly,
            Self::Yearly => Schedule::Yearly,
        }
    }
}

#[poise::command(slash_command)]
pub async fn roles_list(
    ctx: Context<'_>,
    limit: Option<u8>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(async move {
        let Some(guild) = ctx.guild().map(|v| (*v).clone()) else { return loc!(ctx, "cmd-not-in-guild")  };
        let Some(db) = &ctx.data().db else { return loc!(ctx, "database-unreachable") };
        let Ok(elections) = db::queries::elections::get_many(db, guild.id.get(), limit.unwrap_or(10).into()).await
            else { return loc!(ctx, "elections-roles-list-empty") };
    
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_header(vec![
                loc!(ctx, "index"),
                loc!(ctx, "role"),
                loc!(ctx, "scheduled_date"),
                loc!(ctx, "schedule"),
            ]);
    
        for election in elections {
            let mut index = 0;
            if let Some(role) = guild.roles.get(&RoleId::from(election.role.to_u64().unwrap())) {
                index += 1;
                let scheduled_for = match election.scheduled_date {
                    Some(next) => next.to_string(),
                    None => "-".to_string(),
                };
                table.add_row(vec![
                    index.to_string(),
                    role.name.to_string(),
                    scheduled_for,
                    election.schedule.unwrap_or("-".to_string()),
                ]);
            };
        };

        if table.row_count() > 0 {
            loc!(ctx, "elections-roles-list", table: table.to_string() )
        } else {
            loc!(ctx, "elections-roles-list-empty")
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_add(
    ctx: Context<'_>,
    role: Role,
    number_of_positions: Option<i16>,
    schedule: Option<Schedule>,
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

        if let Ok(_) = db::queries::elections::create(
            db,
            role.id.get(),
            guild.id.get(),
            number_of_positions,
            schedule.map(|v| v.into())
        ).await {
            loc!(ctx, "elections-roles-add", role: role.name)
        } else {
            if db::queries::elections::exists(db, role.id.get()).await {
                loc!(ctx, "elections-roles-exists")
            } else { loc!(ctx, "database-oops") }
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_edit(
    ctx: Context<'_>,
    role: Role,
    number_of_positions: Option<i16>,
    schedule: Option<Schedule>,
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

        if let Ok(v) = db::queries::elections::update(
            db,
            role.id.get(),
            number_of_positions,
            schedule.map(|v| v.into())
        ).await {
            loc!(ctx, "elections-roles-edit", role: role.name, number_of_positions: v.limit, schedule: v.schedule.unwrap_or(String::from("-")))
        } else {
            if !db::queries::elections::exists(db, role.id.get()).await {
                loc!(ctx, "elections-not-found", role: role.name)
            } else { loc!(ctx, "database-oops") }
        }
    }.await).await;
    Ok(())
}

#[poise::command(slash_command, required_permissions="ADMINISTRATOR")]
pub async fn roles_remove(
    ctx: Context<'_>,
    role: Role,
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
    
        if let Ok(v) = db::queries::elections::delete(db,role.id.get()).await {
            if v.rows_affected > 0 { 
                loc!(ctx, "elections-roles-delete", role: role.name)
            } else {
                loc!(ctx, "elections-not-found", role: role.name)
            }
        } else { loc!(ctx, "database-oops") }
    }.await).await;
    Ok(())
}
