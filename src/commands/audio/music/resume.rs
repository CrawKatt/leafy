use crate::commands::audio::AudioState;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    user_cooldown = 10,
    guild_only,
)]
pub async fn resume(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;
    let lavalink = &ctx.data().lavalink;

    let Some(player_ctx) = lavalink.get_player_context(guild_id.get()) else {
        ctx.say("No hay nada pausado").await?;
        return Ok(());
    };

    player_ctx.set_pause(false).await?;
    ctx.say("▶️ Reanudando música").await?;

    {
        let mut audio_state = ctx.data().voice_chat_state.lock().await;
        *audio_state = AudioState::Music;
    }

    Ok(())
}