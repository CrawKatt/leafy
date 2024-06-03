use serenity::all::{CreateEmbed, CreateMessage};
use songbird::input::YoutubeDl;
use crate::{HttpKey, location};

use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Audio",
    aliases("p"),
)]
pub async fn play(ctx: Context<'_>, query: String) -> CommandResult {
    let do_search = !query.starts_with("http");
    let url = super::try_get_song(ctx, do_search, query).await?;

    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    super::try_join(ctx, guild).await?;

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .unwrap_log(location!())?
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };
    
    let mut handler = handler_lock.lock().await;
    let src = YoutubeDl::new(http_client, url.clone());

    let message = if handler.queue().is_empty() {
        "Reproduciendo"
    } else {
        "Añadido a la cola"
    };

    let embed = CreateEmbed::new()
        .title(message)
        .description(&url)
        .color(0x00ff_0000);

    let builder = CreateMessage::new().embed(embed);

    ctx.channel_id().send_message(ctx.http(), builder).await?;
    handler.enqueue_input(src.clone().into()).await;
    
    drop(handler);

    Ok(())
}