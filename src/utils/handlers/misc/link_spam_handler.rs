use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::Mutex;
use serenity::all::{ChannelId, GetMessages, Member, Message, UserId};
use poise::serenity_prelude as serenity;
use crate::utils::CommandResult;
use crate::utils::handlers::misc::everyone_case::handle_everyone;

/// Estructura que almacena el id de un usuario,
/// el contenido del mensaje y los canales en los que se ha enviado.
#[derive(Debug)]
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

    async fn add_or_update_message(author_id: UserId, message_content: String, channel_id: ChannelId) {
        let mut message_tracker = MESSAGE_TRACKER.lock().await;
        if let Some(message) = message_tracker.iter_mut().find(|m| m.author_id == author_id && m.message_content == message_content) {
            message.channel_ids.push(channel_id);
        } else {
            // Si el mensaje no existe o el contenido es diferente, borra los mensajes antiguos del autor
            message_tracker.retain(|m| m.author_id != author_id);
            let message = Self::new(author_id, message_content, vec![channel_id]);
            message_tracker.push(message);
        }
    }
}

static MESSAGE_TRACKER: Lazy<Mutex<Vec<MessageTracker>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").map_or(None, |url_re| url_re.find(text).map(|m| m.as_str().to_string()))
}

/// Esta función se encarga de comprobar si un usuario ha enviado el mismo mensaje en
/// 3 canales diferentes y si es así, borra los mensajes y aplica Timeout al autor.
pub async fn spam_checker(
    message_content: String,
    channel_id: ChannelId,
    admin_role_id: &Option<String>,
    member: &mut Member,
    ctx: &serenity::Context,
    time: i64,
    new_message: &Message
) -> CommandResult {
    // Limita el alcance del bloqueo del Mutex
    let author_id = new_message.author.id;
    let mut message_tracker = MESSAGE_TRACKER.lock().await;

    // Comprueba si el último mensaje del usuario es diferente al mensaje actual
    // Nota: No es posible usar let else aquí porque se sale de la función antes de
    // que se pueda obtener otro mensaje
    if let Some(last_message) = message_tracker
        .iter()
        .last()
    {
        // Si el último mensaje es del mismo autor y el contenido es diferente, borra el rastreador de mensajes
        if last_message.author_id == author_id && last_message.message_content != message_content {
            message_tracker.clear();
        }
    }

    // Busca si el mensaje ya existe en el rastreador de mensajes
    let Some(message) = message_tracker
        .iter_mut()
        .find(|m| m.author_id == author_id && m.message_content == message_content) else
    {
        // Si el mensaje no existe, crea un nuevo rastreador de mensajes y añádelo a la lista
        let message = MessageTracker::new(author_id, message_content.clone(), vec![channel_id]);
        message_tracker.push(message);

        return Ok(())
    };

    // Si el mensaje existe, añade el canal a la lista de canales
    message.channel_ids.push(channel_id);
    // println aquí para Debug cuando sea necesario
    println!("Message Tracker: {message_tracker:?}");

    // Comprueba si el usuario ha enviado el mismo mensaje en 3 canales diferentes
    let Some(message) = message_tracker
        .iter()
        .find(|m| m.author_id == author_id && m.message_content == message_content && m.channel_ids.len() >= 3) else
    {
        return Ok(())
    };

    handle_everyone(admin_role_id.to_owned(), member, ctx, time, new_message).await?;
    delete_spam_messages(message, ctx, author_id, message_content).await?;

    // Limpia completamente el rastreador de mensajes para reiniciar el rastreo de mensajes
    message_tracker.retain(|m| m.author_id != author_id);
    drop(message_tracker);

    Ok(())
}

async fn delete_spam_messages(
    message: &MessageTracker,
    ctx: &serenity::Context,
    author_id: UserId,
    message_content: String
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

    Ok(())
}