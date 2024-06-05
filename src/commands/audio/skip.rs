use crate::location;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Audio",
)]
pub async fn skip(ctx: Context<'_>) -> CommandResult {
    let songbird = songbird::get(ctx.serenity_context())
        .await
        .unwrap_log(location!())?;

    let Some(call) = songbird.get(ctx.guild_id().unwrap()) else {
        ctx.say("No estás en un canal de voz").await?;

        return Ok(());
    };

    let _ = call.lock().await.queue().skip();
    
    ctx.say("Se ha saltado la canción").await?;

    Ok(())
}