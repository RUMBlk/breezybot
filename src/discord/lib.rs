use poise::serenity_prelude::{ Role, Member, Guild };

pub async fn has_sufficient_role(guild: &Guild, member: &Member, compare_with: &Role) -> Result<bool, ()> {
    if guild.owner_id == member.user.id { return Ok(true) }
    let Some(highest_role) = guild.member_highest_role(member) else { return Err(()) };
    Ok(*highest_role > *compare_with)
}