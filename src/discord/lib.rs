use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::{ Role, Member };

use crate::Context;

pub async fn role_id_to_role(ctx: Context<'_>, member: &Member, role_id: RoleId) -> Option<Role> {
    let role = role_id.to_role_cached(ctx.cache());
    if role.is_some() { return role } else {
        let guild = member.guild_id.to_partial_guild(ctx.http()).await.ok()?;
        guild.roles.get(&role_id).cloned()
    }
}

pub async fn highest_role(ctx: Context<'_>, member: &Member) -> Option<Role> {
    if let Some((role_id, _position)) = member.highest_role_info(ctx.cache()) {
        role_id_to_role(ctx, member, role_id).await
    } else {
        let mut highest_role = None;
        for role_id in &member.roles {
            role_id_to_role(ctx, &member.clone(), role_id.clone()).await
            .map(|role| {
                if highest_role.is_none() {
                    highest_role = Some(role);
                } else {
                    if role.position > highest_role.clone().unwrap().position {
                        highest_role = Some(role);
                    }
                }
            });
        }
        highest_role
    }
}