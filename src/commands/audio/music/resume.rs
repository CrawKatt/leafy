use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = 10,
    category = "Audio",
)]
pub async fn resume(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };

    let handler = handler_lock.lock().await;

    handler.queue().resume()?;

    ctx.say("Se ha resumido la reproducción").await?;

    drop(handler);

    Ok(())
}