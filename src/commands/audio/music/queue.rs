use poise::CreateReply;
use serenity::all::CreateEmbed;
use crate::handlers::misc::buttons::generate_row;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;
use serenity::futures::future;
use serenity::futures::StreamExt;
use std::fmt::Write;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Audio",
    user_cooldown = 10,
    aliases("q"),
)]
pub async fn queue(ctx: Context<'_>) -> CommandResult {
    let guild_id = ctx.guild_id().into_result()?;
    let lavalink = &ctx.data().lavalink;

    let Some(player_ctx) = lavalink.get_player_context(guild_id.get()) else {
        ctx.say("No estoy conectado a un canal de voz").await?;
        return Ok(());
    };

    let player_data = player_ctx.get_player().await?;
    let queue = player_ctx.get_queue();
    
    let max = queue.get_count().await?.min(10);
    
    let queue_list = queue
        .enumerate()
        .take_while(|(idx, _)| future::ready(*idx < max))
        .map(|(idx, x)| {
            format!(
                "{}. {} - {}",
                idx + 1,
                x.track.info.author,
                x.track.info.title
            )
        })
        .collect::<Vec<_>>()
        .await
        .join("\n");

    let mut description = String::new();
    if let Some(track) = player_data.track {
        description.push_str("**Ahora reproduciendo:**\n");
        write!(description, "{} - {}\n\n", track.info.title, track.info.author)?;
    }

    if !queue_list.is_empty() {
        description.push_str("**En cola:**\n");
        description.push_str(&queue_list);
    }

    if description.is_empty() {
        ctx.say("No hay canciones en cola").await?;
        return Ok(());
    }
    
    let buttons = generate_row(player_data.paused);
    let components = vec![buttons];
    
    let embed = CreateEmbed::default()
        .title("Canciones en cola")
        .description(description)
        .color(0x0000_ff00);

    let builder = CreateReply::default()
        .embed(embed)
        .components(components);
    ctx.send(builder).await?;

    Ok(())
}