use serenity::all::GetMessages;

use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    guild_only,
)]
pub async fn leave(ctx: Context<'_>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let msg = messages.first().into_result()?;

    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

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