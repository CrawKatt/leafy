use serenity::all::GetMessages;
use crate::location;

use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    guild_only,
    track_edits
)]
pub async fn join(ctx: Context<'_>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let msg = messages.first().unwrap_log(location!())?;

    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

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