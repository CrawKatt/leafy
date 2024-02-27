use poise::CreateReply;
use tokio::time::Instant;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Info",
    guild_only,
    ephemeral
)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    let before = Instant::now();
    let message = poise::say_reply(ctx, "Pinging...").await?;
    let latency = before.elapsed();
    let new_message = format!("Pong! La latencia del bot es de {} ms", latency.as_millis());
    let reply = CreateReply::default().content(new_message);

    message.edit(ctx, reply).await?;

    Ok(())
}