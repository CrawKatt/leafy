use poise::serenity_prelude as serenity;
use reqwest::Url;
use serenity::all::{ChannelId, MessageId};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::location;

use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::utils::embeds::{send_embed, send_embed_with_attachment};

pub async fn handler(ctx: &serenity::Context, channel_id: &ChannelId, deleted_message_id: &MessageId) -> CommandResult {
    let database_info = MessageData::get_message_data(deleted_message_id).await?;
    let Some(database_message) = database_info else { return Ok(()) };

    handle_message(ctx, channel_id, &database_message).await?;
    handle_audio(ctx, deleted_message_id, database_message, channel_id).await?;

    Ok(())
}

async fn handle_message(ctx: &serenity::Context, channel_id: &ChannelId, database_message: &MessageData) -> CommandResult {
    let message_content = database_message.message_content.clone();
    let message_channel_id = database_message.channel_id;
    let author_id = database_message.author_id;

    // Si el mensaje está vacío, se asume que se trata de un posible mensaje de audio
    if message_content.is_empty() {
        return Ok(())
    }

    // Obtener el canal de logs de la base de datos
    let result_database = database_message.guild_id.unwrap_log(location!())?;
    let log_channel = GuildData::verify_data(result_database).await?
        .into_result()?
        .channels
        .logs
        .into_result()?
        .parse::<ChannelId>()?;

    if channel_id == &log_channel { return Ok(()) }
    send_embed(ctx, result_database, log_channel, &message_channel_id, author_id, &message_content).await?;

    Ok(())
}

async fn handle_audio(ctx: &serenity::Context, deleted_message_id: &MessageId, database_message: MessageData, channel_id: &ChannelId) -> CommandResult {
    let audio_info = MessageData::get_audio_data(deleted_message_id).await?;
    let Some(audio_info) = audio_info else { return Ok(()) };

    // Descargar el archivo de audio
    let attachment_url = audio_info.message_content;
    let url = Url::parse(&attachment_url)?;
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let filename = format!("/tmp/{}", attachment_url.split('/').next_back().unwrap_or_default());
    let mut out = File::create(&filename).await?;
    out.write_all(&bytes).await?;

    let result_database = database_message.guild_id.into_result()?;
    let log_channel = GuildData::verify_data(result_database).await?
        .into_result()?
        .channels
        .logs
        .into_result()?
        .parse::<ChannelId>()?;

    if channel_id == &log_channel { return Ok(()) }
    send_embed_with_attachment(ctx, result_database, log_channel, &database_message.channel_id, database_message.author_id, &filename).await?;

    Ok(())
}