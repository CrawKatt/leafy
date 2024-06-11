use crate::commands::audio::get_guild_id_and_channel_id;

use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    user_cooldown = 10,
    guild_only,
)]
pub async fn leave(ctx: Context<'_>) -> CommandResult {
    let (guild_id, channel_id) = get_guild_id_and_channel_id(ctx).await?;

    if channel_id.is_none() {
        ctx.say("No estoy en un canal de voz para salir").await?;
        return Ok(())
    }

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    manager.remove(guild_id).await?;

    Ok(())
}