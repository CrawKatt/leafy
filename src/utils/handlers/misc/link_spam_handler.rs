use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::Mutex;
use serenity::all::{ChannelId, CreateEmbedAuthor, CreateMessage, GetMessages, GuildId, Member, Message, UserId};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;
use crate::commands::setters::GuildData;
use crate::utils::CommandResult;
use crate::utils::handlers::misc::everyone_case::handle_everyone;
use crate::utils::misc::debug::UnwrapLog;

#[derive(Debug, Default)]
struct MessageTracker {
    author_id: UserId,
    message_content: String,
    channel_ids: Vec<ChannelId>,
}

impl MessageTracker {
    fn new(author_id: UserId, message_content: String, channel_ids: Vec<ChannelId>) -> Self {
        Self {
            author_id,
            message_content,
            channel_ids,
        }
    }
}

static MESSAGE_TRACKER: Lazy<Mutex<Vec<MessageTracker>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").map_or(None, |url_re| url_re.find(text).map(|m| m.as_str().to_string()))
}

pub async fn spam_checker(
    message_content: String,
    channel_id: ChannelId,
    admin_role_id: &Option<String>,
    ctx: &serenity::Context,
    time: i64,
    new_message: &Message,
    guild_id: GuildId
) -> CommandResult {
    let author_id = new_message.author.id;
    let mut member = guild_id.member(&ctx.http, new_message.author.id).await?;
    let mut message_tracker = MESSAGE_TRACKER.lock().await;

    if let Some(last_message) = message_tracker
        .iter()
        .last()
    {
        if last_message.author_id == author_id && last_message.message_content != message_content {
            message_tracker.clear();
        }
    }

    let message = if let Some(message) = message_tracker
        .iter_mut()
        .find(|m| m.author_id == author_id && m.message_content == message_content)
    {
        // Si el mensaje existe y el canal no está en la lista de canales, añade el canal a la lista de canales
        if message.channel_ids.contains(&channel_id) {
            // Si el mensaje se repite en el mismo canal, borra el vector
            println!("Message repeated in the same channel, clearing the vector");
            message_tracker.clear();

            return Ok(());
        }
        message.channel_ids.push(channel_id);

        message
    } else {
        // Si el mensaje no existe, crea un nuevo rastreador de mensajes y añádelo a la lista
        let message = MessageTracker::new(author_id, message_content.clone(), vec![channel_id]);
        message_tracker.push(message);
        message_tracker.last_mut().unwrap_log("No se pudo obtener el último mensaje", module_path!(), line!())?
    };

    if message.channel_ids.len() >= 3 {
        handle_everyone(admin_role_id.to_owned(), &mut member, ctx, time, new_message).await?;
        delete_spam_messages(message, ctx, author_id, message_content, guild_id).await?;

        // Limpia completamente el rastreador de mensajes para reiniciar el rastreo de mensajes
        message_tracker.retain(|m| m.author_id != author_id);
    }
    println!("Tracker: {message_tracker:#?}");

    drop(message_tracker);

    Ok(())
}

async fn delete_spam_messages(
    message: &MessageTracker,
    ctx: &serenity::Context,
    author_id: UserId,
    message_content: String,
    guild_id: GuildId
) -> CommandResult {
    // Borra cada mensaje individualmente
    for channel_id in &message.channel_ids {
        let channel = channel_id.to_channel(ctx).await?;
        let serenity::Channel::Guild(channel) = channel else {
            return Ok(())
        };

        let messages = channel.messages(&ctx.http, GetMessages::new()).await?;
        for message in messages {
            if message.author.id == author_id && message.content == message_content {
                message.delete(&ctx.http).await?;
            }
        }
    }

    let data = GuildData::get_log_channel(guild_id).await?;
    let log_channel= data.unwrap_log("No se pudo obtener el canal de registro", module_path!(), line!())?.log_channel_id;
    let author_user = author_id.to_user(&ctx.http).await?;
    let username = &author_user.name;
    let embed = CreateEmbed::default()
        .title("Spam detectado")
        .author(CreateEmbedAuthor::new(username)
            .icon_url(author_user.avatar_url().unwrap_or_else(|| author_user.default_avatar_url())))
        .description(format!("El usuario <@{author_id}> Es sospechoso de enviar spam en el servidor.\nMensaje: {message_content}"))
        .color(0x00ff_0000);

    log_channel.send_message(&ctx.http, CreateMessage::default().embed(embed)).await?;

    Ok(())
}