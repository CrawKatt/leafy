use crate::location;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Audio",
)]
pub async fn pause(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap_log(location!())?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };

    let handler = handler_lock.lock().await;
    handler.queue().pause()?;
    ctx.say("Se ha pausado la canción").await?;

    drop(handler);

    Ok(())
}