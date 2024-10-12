use crate::handlers::error::handler;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::{CommandResult, Context};
use poise::async_trait;
use songbird::{input, Event, EventContext, EventHandler, TrackEvent};
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::utils::metadata::build_embed;

struct FileCleaner {
    paths: Vec<String>,
}

#[async_trait]
impl EventHandler for FileCleaner {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        for path in &self.paths {
            if Path::new(path).exists() {
                fs::remove_file(path).unwrap_or_else(|why| {
                    println!("No se pudo eliminar el archivo: {why}");
                });
            }
        }
        None
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    on_error = "handler",
    user_cooldown = 10,
    category = "Audio",
    aliases("p"),
)]
pub async fn play(
    ctx: Context<'_>,
    #[rest]
    query: String
) -> CommandResult {
    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    super::try_join(ctx, guild).await?;

    let author_name = ctx.author_member().await.into_result()?.distinct();
    let author_face = ctx.author_member().await.into_result()?.face();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };

    let message = ctx.say("Descargando...").await?;

    // Descargar el archivo de audio con yt-dlp
    let cookies_path = "~/cookies.txt";
    let output_path = format!("/tmp/{}.mp3", uuid::Uuid::new_v4());
    let json_path = format!("{output_path}.info.json");
    let limit_rate = "500K";

    let status = Command::new("yt-dlp")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--add-metadata")
        .arg("--write-info-json")
        .arg("--limit-rate")
        .arg(limit_rate)
        .arg("-o")
        .arg(&output_path)
        .arg("--cookies")
        .arg(cookies_path)
        .arg(&query)
        .status()
        .expect("No se pudo ejecutar yt-dlp");

    if !status.success() {
        ctx.say("Error al descargar el audio").await?;
        return Ok(())
    }

    let mut handler = handler_lock.lock().await;
    let source = input::File::new(output_path.clone());
    let track_handle = handler.enqueue_input(source.into()).await;
    track_handle.add_event(
        Event::Track(TrackEvent::End),
        FileCleaner { paths: vec![output_path.clone(), json_path.clone()] },
    )?;

    build_embed(&ctx, &json_path, &author_name, &author_face).await?;
    message.delete(ctx).await?;
    
    drop(handler);

    Ok(())
}