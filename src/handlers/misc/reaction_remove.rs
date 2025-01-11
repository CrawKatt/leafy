use poise::serenity_prelude as serenity;
use regex::Regex;
use serenity::all::{Reaction, RoleId};

use crate::utils::config::AutoRole;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::CommandResult;

pub async fn handler(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    autorole_remove(ctx, add_reaction).await?;
    Ok(())
}

pub async fn autorole_remove(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    let guild_id = add_reaction.guild_id.ok_or("error")?;
    let emoji = add_reaction.emoji.to_string();
    let emoji_id = Regex::new(r"^<a?:\w+:(\d+)>")?
        .captures(&emoji)
        .into_result()?
        .get(1)
        .into_result()?
        .as_str()
        .to_string();

    let data = AutoRole::get_assignments(guild_id).await?;
    let Some(data) = data else {
        println!("[ERROR]: No se encontraron asignaciones de autoroles para este servidor.");
        return Ok(());
    };

    if let Some(assignment) = data
        .assignments
        .iter()
        .find(|a| a.emoji_id == emoji_id)
    {
        let role_id: RoleId = assignment.role.parse()?;
        let user = add_reaction.user_id.ok_or("UserID not found")?;
        let member = guild_id.member(ctx, user).await?;
        member.remove_role(ctx, role_id).await?;
    }

    Ok(())
}