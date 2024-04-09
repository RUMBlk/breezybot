use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::{ Role, Member };

use crate::Context;

pub async fn role_id_to_role(ctx: Context<'_>, member: &Member, role_id: RoleId) -> Option<Role> {
    match role_id.to_role_cached(ctx.cache()) {
        Some(role) => Some(role),
        None => {
            match member.guild_id.to_partial_guild(ctx.http()).await {
                Ok(guild) => guild.roles.get(&role_id).cloned(),
                Err(_) => None,
            }
        },
    }
}

pub async fn highest_role(ctx: Context<'_>, member: &Member) -> Option<Role> {
    match member.highest_role_info(ctx.cache()) {
        Some((role_id, _position)) => {
            role_id_to_role(ctx, member, role_id).await
        },
        None => {
            let mut highest_role = None;
            for role_id in &member.roles {
                if let Some(role) = role_id_to_role(ctx, &member.clone(), role_id.clone()).await {
                    if highest_role == None {
                        highest_role = Some(role);
                    } else {
                        if role.position > highest_role.clone().unwrap().position {
                            highest_role = Some(role);
                        }
                    }
                }
            }
            highest_role
        },
    }
}