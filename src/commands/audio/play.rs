use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
use songbird::input::YoutubeDl;
use crate::{HttpKey, location};
use crate::commands::audio::queue::AuxMetadataKey;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::handlers::error::err_handler;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    on_error = "err_handler",
    user_cooldown = 10,
    category = "Audio",
    aliases("p"),
)]
pub async fn play(ctx: Context<'_>, query: String) -> CommandResult {
    let do_search = !query.starts_with("http");
    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    super::try_join(ctx, guild).await?;

    let author_name = ctx.author_member()
        .await
        .into_result()?
        .distinct();

    let author_face = ctx.author_member()
        .await
        .into_result()?
        .face();

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

    let message = ctx.say("Buscando...").await?;
    
    // Necesario para bypassear el baneo de YouTube a Bots
    // (No utilizar cookies de cuentas de Google personales)
    let cookies = format!("--cookies ~/cookies.txt {query}");
    let mut handler = handler_lock.lock().await;
    let source = if do_search {
        YoutubeDl::new_search(http_client, cookies)
    } else {
        YoutubeDl::new(http_client, cookies)
    };
    
    let mut src: songbird::input::Input = source.into();
    
    // Obtener la metadata auxiliar de la pista, como el título y la miniatura
    let aux_metadata = src.aux_metadata().await?;
    let title = aux_metadata.title.clone().into_result()?;
    let thumbnail = aux_metadata.thumbnail.clone().into_result()?;
    
    // Insertar la pista en la cola de reproducción
    let track = handler.enqueue_input(src).await;

    // Insertar la metadata en el TypeMap, para poder acceder
    // a la metadata de la cola de reproducción
    let mut map = track.typemap().write().await;
    map.entry::<AuxMetadataKey>().or_insert(aux_metadata);

    let song_name = if handler.queue().is_empty() { format!("Reproduciendo {title}") } else { format!("{title} Añadido a la cola") };

    message.delete(ctx).await?;
    let desc = format!("**Solicitado por:** {author_name}");
    let embed = CreateEmbed::new()
        .title(song_name)
        .author(CreateEmbedAuthor::new(author_name)
            .icon_url(author_face))
        .description(desc)
        .thumbnail(thumbnail)
        .color(0x00ff_0000);

    let builder = CreateMessage::new().embed(embed);
    ctx.channel_id().send_message(ctx.http(), builder).await?;
    
    // Liberar el bloqueo del manejador y del map para evitar fugas de memoria
    drop(handler);
    drop(map);

    Ok(())
}