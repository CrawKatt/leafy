use crate::utils::config::GuildData;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::CommandResult;
use crate::DB;
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, CreateEmbedAuthor, CreateMessage, GetMessages, GuildId, Message, UserId};
use serenity::builder::CreateEmbed;
use std::sync::Arc;
use bon::Builder;
use surrealdb::Result as SurrealResult;

use crate::handlers::misc::everyone_case::handle_everyone;

/// # Estructura de rastreador de mensajes
/// Representa cada mensaje rastreado, con autor, contenido, canales y tiempo.
#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
struct MessageTracker {
    author_id: UserId,
    message_content: Arc<String>,
    channel_ids: Vec<ChannelId>,
    last_message_time: DateTime<Utc>,
}

impl MessageTracker {
    /// Inserta un nuevo message tracker en la base de datos.
    async fn insert(&self) -> SurrealResult<()> {
        let _created: Option<Self> = DB
            .create("message_tracker")
            .content(self.clone())
            .await?;

        Ok(())
    }

    /// Recupera el tracker de mensajes de un autor por ID.
    /// - Si el último mensaje en la Base de Datos es 5 segundos más antiguo que
    ///   el mensaje recibido, se borra el tracker para limpiar 
    /// - Se usa `.take(1)` para tomar el resultado de la query `SELECT`
    async fn get_by_author(author_id: UserId) -> SurrealResult<Vec<Self>> {
        let threshold = Utc::now() - chrono::Duration::seconds(5);

        let sql_query = "
            DELETE FROM message_tracker WHERE last_message_time < $threshold;
            SELECT * FROM message_tracker WHERE author_id = $author_id;
        ";

        let trackers = DB.query(sql_query)
            .bind(("threshold", threshold.to_rfc3339()))
            .bind(("author_id", author_id.to_string()))
            .await?
            .take(1)?;

        Ok(trackers)
    }

    /// Actualiza el message tracker en la base de datos.
    async fn update(self) -> SurrealResult<()> {
        let sql_query = "UPDATE message_tracker SET channel_ids = $channel_ids, last_message_time = $last_message_time WHERE author_id = $author_id AND message_content = $message_content";
        DB.query(sql_query)
            .bind(("channel_ids", self.channel_ids))
            .bind(("last_message_time", self.last_message_time.to_rfc3339()))
            .bind(("author_id", self.author_id.to_string()))
            .bind(("message_content", self.message_content)).await?;

        Ok(())
    }

    /// Elimina el message tracker de un autor por ID.
    async fn delete_by_author(author_id: UserId) -> SurrealResult<()> {
        let sql_query = "DELETE FROM message_tracker WHERE author_id = $author_id";
        DB.query(sql_query)
            .bind(("author_id", author_id.to_string()))
            .await?;
        Ok(())
    }
}

/// # Esta función extrae un enlace de un mensaje
///
/// - Si el mensaje contiene un enlace, devuelve el enlace.
pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").ok()?.find(text).map(|m| m.as_str().to_string())
}

/// # Verifica si un mensaje es spam.
pub async fn spam_checker(
    message_content: &Arc<String>,
    channel_id: ChannelId,
    admin_role_id: Option<&Vec<String>>,
    ctx: &serenity::Context,
    time: i64,
    new_message: &Message,
    guild_id: GuildId,
) -> CommandResult {
    let author_id = new_message.author.id;
    let mut trackers = MessageTracker::get_by_author(author_id).await?;
    let existing_tracker = trackers.iter_mut().find(|tracker| tracker.message_content == *message_content);
    let Some(tracker) = existing_tracker else {
        let new_tracker = MessageTracker::builder()
            .author_id(author_id)
            .message_content(message_content.clone())
            .channel_ids(vec![channel_id])
            .last_message_time(Utc::now())
            .build();

        new_tracker.insert().await?;
        trackers.push(new_tracker);

        return Ok(())
    };

    tracker.last_message_time = Utc::now();
    if !tracker.channel_ids.contains(&channel_id) {
        tracker.channel_ids.push(channel_id);
    }
    tracker.clone()
        .update()
        .await?;

    if trackers.iter().any(|t| t.channel_ids.len() >= 3) {
        let mut member = guild_id.member(&ctx.http, author_id).await?;
        handle_everyone(admin_role_id, &mut member, ctx, time, new_message).await?;
        delete_spam_messages(trackers.last().unwrap(), ctx, author_id, message_content.clone(), guild_id).await?;

        MessageTracker::delete_by_author(author_id).await?;
    }

    Ok(())
}

/// Borra los mensajes de spam.
async fn delete_spam_messages(
    message: &MessageTracker,
    ctx: &serenity::Context,
    author_id: UserId,
    message_content: Arc<String>,
    guild_id: GuildId,
) -> CommandResult {
    for channel_id in &message.channel_ids {
        let channel = channel_id.to_channel(ctx).await?;
        let serenity::Channel::Guild(channel) = channel else {
            continue;
        };

        let messages = channel.messages(&ctx.http, GetMessages::new()).await?;
        for message in messages {
            if message.author.id == author_id && message.content == *message_content {
                message.delete(&ctx.http).await?;
            }
        }
    }

    create_embed(ctx, guild_id, author_id, &message_content).await?;
    Ok(())
}

/// Crea un `Embed` para registrar la detección de spam.
async fn create_embed(
    ctx: &serenity::Context,
    guild_id: GuildId,
    author_id: UserId,
    message_content: &str,
) -> CommandResult {
    let log_channel = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channels
        .logs
        .into_result()?
        .parse::<ChannelId>()?;

    let author_user = author_id.to_user(&ctx.http).await?;
    let embed = CreateEmbed::default()
        .title("⚠️ Spam detectado")
        .author(CreateEmbedAuthor::new(&author_user.name).icon_url(author_user.face()))
        .description(format!("El usuario <@{author_id}> fue detectado enviando spam.\nMensaje: {message_content}"))
        .color(0x00FF_0000);

    log_channel.send_message(&ctx.http, CreateMessage::default().embed(embed)).await?;
    Ok(())
}