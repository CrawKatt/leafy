use poise::serenity_prelude as serenity;
use reqwest::Url;
use serenity::all::{ChannelId, MessageId, UserId};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::DB;
use crate::utils::Error;
use crate::utils::MessageData;
use crate::commands::setters::GuildData;
pub async fn delete_message_handler(ctx: &serenity::Context, channel_id: &ChannelId, deleted_message_id: &MessageId) -> Result<(), Error> {
    let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
    let database_info: Option<MessageData> = DB
        .query(sql_query)
        .bind(("message_id", deleted_message_id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(database_message) = database_info else {
        return Ok(())
    };

    let message_content = database_message.message_content.clone();
    let message_channel_id = database_message.channel_id;
    let author_id = database_message.author_id;

    if message_content.is_empty() {
        let sql_query = "SELECT * FROM audio WHERE message_id = $message_id";
        let audio_info: Option<MessageData> = DB
            .query(sql_query)
            .bind(("message_id", deleted_message_id)) // pasar el valor
            .await?
            .take(0)?;

        if audio_info.is_some() {
            handle_audio(ctx, deleted_message_id, &database_message, channel_id).await?;
            return Ok(())
        }

    }

    // Obtener el canal de logs de la base de datos
    let log_channel_database = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(log_channel_database)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_or_default().log_channel_id;
    if channel_id == &log_channel {
        return Ok(());
    }

    // variable que busca la mención en el menssage_content si existe
    let mention = message_content.find("<@");

    // convertir el mention en un objeto User
    let Some(_) = mention else {
        crate::utils::embeds::send_embed(ctx,log_channel, &message_channel_id, author_id, &message_content).await;
        return Ok(());
    };

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    let user = UserId::new(user_id);
    let user_mentioned = user.to_user(&ctx.http).await?;

    crate::utils::embeds::send_embed_if_mention(ctx,log_channel, &message_channel_id, author_id, &message_content,user_mentioned).await;

    Ok(())
}

async fn handle_audio(ctx: &serenity::Context, deleted_message_id: &MessageId, database_message: &MessageData, channel_id: &ChannelId) -> Result<(), Error> {
    let sql_query = "SELECT * FROM audio WHERE message_id = $message_id";
    let audio_info: Option<MessageData> = DB
        .query(sql_query)
        .bind(("message_id", deleted_message_id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(audio_info) = audio_info else {
        return Ok(());
    };

    let attachment_url = audio_info.message_content;
    let url = Url::parse(&attachment_url)?;
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let filename = format!("/tmp/{}", attachment_url.split("/").last().unwrap_or_default());
    let mut out = File::create(&filename).await?;
    out.write_all(&bytes).await?;

    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_or_default().log_channel_id;
    if channel_id == &log_channel {
        return Ok(());
    }

    crate::utils::embeds::send_embed_with_attachment(ctx,log_channel, &database_message.channel_id, database_message.author_id, &filename).await;

    Ok(())
}