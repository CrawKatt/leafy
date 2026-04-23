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
pub async fn pause(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;
    let lavalink = &ctx.data().lavalink;

    let Some(player_ctx) = lavalink.get_player_context(guild_id.get()) else {
        ctx.say("No hay nada reproduciéndose").await?;
        return Ok(());
    };

    player_ctx.set_pause(true).await?;
    ctx.say("⏸️ Música pausada").await?;

    {
        let mut audio_state = ctx.data().voice_chat_state.lock().await;
        *audio_state = AudioState::Idle;
    }

    Ok(())
}