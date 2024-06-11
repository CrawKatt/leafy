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
pub async fn join(ctx: Context<'_>) -> CommandResult {
    let (guild_id, channel_id) = get_guild_id_and_channel_id(ctx).await?;

    let Some(connect_to) = channel_id else {
        ctx.say("No estás en un canal de voz para unirme").await?;
        return Ok(())
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    manager.join(guild_id, connect_to).await?;
    
    Ok(())
}