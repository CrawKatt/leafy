use serenity::all::{EmojiId, Role};

use crate::utils::config::AutoRole;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_autorole(
    ctx: Context<'_>,
    emoji: String,
    role: Role,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    
    let regex = regex::Regex::new(r"^<a?:\w+:(\d+)>$")?;
    let emoji_name = regex
        .captures(&emoji)
        .unwrap()
        .get(0)
        .unwrap()
        .as_str()
        .split(':')
        .collect::<Vec<&str>>()[1];

    let emoji_id = regex.captures(&emoji).unwrap().get(1).unwrap().as_str().parse::<EmojiId>()?;

    AutoRole::add_assignment(guild_id, emoji_id.to_string(), emoji_name.to_string(), role.id.to_string()).await?;
    ctx.say(format!(
        "Se ha establecido el rol {} como autorol para el emoji {}",
        role.name, emoji
    )).await?;

    Ok(())
}