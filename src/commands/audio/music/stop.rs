use crate::commands::audio::AudioState;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    user_cooldown = 10,
    guild_only,
)]
pub async fn stop(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;
    let lavalink = &ctx.data().lavalink;

    let Some(player_ctx) = lavalink.get_player_context(guild_id.get()) else {
        ctx.say("No hay nada reproduciéndose").await?;
        return Ok(());
    };

    player_ctx.stop_now().await?;
    player_ctx.get_queue().clear()?;
    ctx.say("⏹️ Música detenida y cola vaciada").await?;

    {
        let mut audio_state = ctx.data().voice_chat_state.lock().await;
        *audio_state = AudioState::Idle;
    }

    Ok(())
}