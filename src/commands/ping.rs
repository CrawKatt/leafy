use tokio::time::Instant;
use crate::utils::{Context, Error};

#[poise::command(
prefix_command,
slash_command,
category = "Info",
guild_only,
ephemeral
)]
/// Check if yours truly is alive and well.
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start_time = Instant::now();

    let (mut shard_ids, mut shard_stages, mut shard_latencies) =
        (Vec::new(), Vec::new(), Vec::new());

    let runners = ctx.framework().shard_manager.runners.lock().await;
    let runners_iter = runners.iter();

    for (id, runner) in runners_iter {
        let stage = runner.stage;
        let latency = runner.latency;

        shard_ids.push(id);
        shard_stages.push(stage);
        shard_latencies.push(latency);
    }

    let elapsed_time = start_time.elapsed();
    let ping = elapsed_time.as_millis();

    ctx.say(format!("Pong. La latencia del Bot es de: **{ping}ms**")).await?;

    Ok(())
}