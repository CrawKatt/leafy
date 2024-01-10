//use poise::serenity_prelude as poise_serenity;
//use poise::serenity::model::id::ChannelId;

//use serenity::model::prelude::ChannelId;
//use crate::utils::error::{CommandResult, Context};

// ID del canal al que enviar los mensajes eliminados
//const LOG_CHANNEL_ID: u64 = 1193595925503942688;

/*
#[poise::event]
async fn message_delete(
    ctx: Context<'_>,
    deleted_message: &poise_serenity::model::channel::Message,
) -> CommandResult {
    // Crear un canal a partir de la ID
    let log_channel = ChannelId(LOG_CHANNEL_ID);

    // Formatear la información del mensaje eliminado
    let log_message = format!(
        "Mensaje eliminado:\n{}\nAutor: {}",
        deleted_message.content,
        deleted_message.author.name
    );

    // Enviar la información del mensaje eliminado al canal de log
    log_channel.say(&ctx.discord().http, &log_message).await?;

    Ok(())
}
*/