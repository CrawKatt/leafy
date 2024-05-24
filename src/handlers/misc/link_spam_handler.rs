use std::sync::{Arc, LazyLock};
use std::time::Instant;

use poise::serenity_prelude as serenity;
use regex::Regex;
use serenity::all::{ChannelId, CreateEmbedAuthor, CreateMessage, GetMessages, GuildId, Message, UserId};
use serenity::builder::CreateEmbed;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::utils::CommandResult;
use crate::handlers::misc::everyone_case::handle_everyone;
use crate::utils::config::GuildData;
use crate::utils::debug::IntoUnwrapResult;

/// # Estructura de rastreador de mensajes
/// 
/// - Almacena el ID del autor del mensaje
/// - Almacena el contenido del mensaje
/// - Almacena los IDs de los canales
/// - Almacena el tiempo del último mensaje
#[derive(Debug)]
struct MessageTracker {
    author_id: UserId,
    message_content: Arc<String>,
    channel_ids: Vec<ChannelId>,
    last_message_time: Instant,
}

/// Implementación de la estructura de rastreador de mensajes
/// 
/// - Implementa un método para crear un rastreador de mensajes por defecto
/// - Implementa un método para modificar el ID del autor del mensaje
/// - Implementa un método para modificar el contenido del mensaje
/// - Implementa un método para modificar los IDs de los canales
/// - Se sigue el patrón de diseño Builder
impl MessageTracker {
    /// # Default implementado manualmente
    /// 
    /// - Es necesario implementar `Default` manualmente
    /// ya que no puede establecer un valor por defecto para `Instant::now()`
    pub fn default() -> Self {
        Self {
            author_id: UserId::default(),
            message_content: Arc::new(String::default()),
            channel_ids: Vec::new(),
            last_message_time: Instant::now(),
        }
    }

    /// # Modifica el ID del autor del mensaje
    /// 
    /// - Modifica el ID del autor del mensaje
    /// El patrón de diseño Builder consiste en mutar `self` y devolver `Self`
    /// con el campo modificado por el valor proporcionado como argumento 
    /// - Devuelve la estructura de rastreador de mensajes
    const fn author_id(mut self, author_id: UserId) -> Self {
        self.author_id = author_id;
        self
    }

    /// # Modifica el contenido del mensaje
    ///
    /// El patrón de diseño Builder consiste en mutar `self` y devolver `Self`
    /// con el campo modificado por el valor proporcionado como argumento 
    /// - Devuelve la estructura de rastreador de mensajes
    fn message_content(mut self, message_content: Arc<String>) -> Self {
        self.message_content = message_content;
        self
    }

    /// # Modifica los IDs de los canales
    ///
    /// El patrón de diseño Builder consiste en mutar `self` y devolver `Self`
    /// con el campo modificado por el valor proporcionado como argumento
    /// - Devuelve la estructura de rastreador de mensajes
    fn channel_ids(mut self, channel_ids: Vec<ChannelId>) -> Self {
        self.channel_ids = channel_ids;
        self
    }
}

/// # Rastreador de mensajes
/// 
/// - Almacena un vector de rastreadores de mensajes
/// - Se utiliza un `LazyLock` para evitar problemas de concurrencia
/// - Se utiliza un Mutex para evitar problemas de concurrencia
/// - Se utiliza un vector para almacenar los rastreadores de mensajes
/// - Se utiliza static para almacenar el rastreador de 
/// mensajes de forma global y evitar la pérdida de datos
static MESSAGE_TRACKER: LazyLock<Mutex<Vec<MessageTracker>>> = LazyLock::new(|| {
    Mutex::new(Vec::new())
});

/// # Esta función extrae un enlace de un mensaje
///
/// - Si el mensaje contiene un enlace, devuelve el enlace
pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").map_or(None, |url_re| url_re.find(text).map(|m| m.as_str().to_string()))
}

/// # Esta función comprueba si un mensaje es spam
///
/// - Si el mensaje se repite en el mismo canal, borra el vector
/// - Si el mensaje no existe, crea un nuevo rastreador de mensajes y añádelo a la lista
/// - Si el siguiente mensaje no coincide con el mensaje anterior, borra el vector
/// - Si se detecta spam, se envía un mensaje al canal de registro, se borran los mensajes de spam y se limpia el vector
pub async fn spam_checker(
    message_content: &Arc<String>,
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
        if last_message.author_id == author_id && last_message.message_content != *message_content {
            message_tracker.clear();
        }
    }

    let message = if let Some(message) = message_tracker
        .iter_mut()
        .find(|m| m.author_id == author_id && m.message_content == *message_content)
    {
        // Inicializa el tiempo del último mensaje
        message.last_message_time = Instant::now();

        // Si el mensaje existe y el canal no está en la lista de canales, añade el canal a la lista de canales
        if message.channel_ids.contains(&channel_id) {
            // Si el mensaje se repite en el mismo canal, borra el vector
            // Debug: println!("Message repeated in the same channel, clearing the vector");
            message_tracker.clear();

            return Ok(());
        }
        message.channel_ids.push(channel_id);

        message
    } else {
        // Si el mensaje no existe, crea un nuevo rastreador de mensajes y añádelo a la lista
        let message = MessageTracker::default()
            .author_id(author_id)
            .message_content(message_content.clone())
            .channel_ids(vec![channel_id]);

        message_tracker.push(message);
        message_tracker.last_mut().into_result()?
    };

    if message.channel_ids.len() >= 3 {
        handle_everyone(admin_role_id.to_owned(), &mut member, ctx, time, new_message).await?;
        delete_spam_messages(message, ctx, author_id, message_content.clone(), guild_id).await?;

        // Limpia completamente el rastreador de mensajes para reiniciar el rastreo de mensajes
        message_tracker.retain(|m| m.author_id != author_id);
    }
    // Debug: println!("Tracker: {message_tracker:#?}");

    drop(message_tracker);

    Ok(())
}

/// # Esta función borra los mensajes de spam
/// 
/// - Borra cada mensaje individualmente en donde 
/// el autor del mensaje y el contenido del mensaje coinciden
async fn delete_spam_messages(
    message: &MessageTracker,
    ctx: &serenity::Context,
    author_id: UserId,
    message_content: Arc<String>,
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
            if message.author.id == author_id && message.content == *message_content {
                message.delete(&ctx.http).await?;
            }
        }
    }

    create_embed(ctx, guild_id, author_id, &message_content).await?;

    Ok(())
}

/// # Esta función crea un `Embed` para enviar un mensaje al canal de Logs
/// 
/// El Log contiene
/// - El nombre del autor del mensaje
/// - La URL de la imagen del autor del mensaje
/// - El contenido del mensaje
/// - El color del mensaje
/// - Los canales donde se enviaron los mensajes de spam
async fn create_embed(
    ctx: &serenity::Context,
    guild_id: GuildId,
    author_id: UserId,
    message_content: &str,
) -> CommandResult {

    let log_channel = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channel_config
        .log_channel_id
        .into_result()?
        .parse::<ChannelId>()?;

    let author_user = author_id.to_user(&ctx.http).await?;
    let author_member = guild_id.member(&ctx.http, author_id).await?;
    let username = author_member.distinct();
    let embed = CreateEmbed::default()
        .title("⚠️ Spam detectado")
        .author(CreateEmbedAuthor::new(username)
            .icon_url(author_user.face()))
        .description(format!("El usuario <@{author_id}> Es sospechoso de enviar spam en el servidor.\nMensaje: {message_content}"))
        .color(0x00ff_0000);
    let builder = CreateMessage::default().embed(embed);

    log_channel.send_message(&ctx.http, builder).await?;

    Ok(())
}

/// # Esta función limpia el rastreador de mensajes
/// 
/// - Limpia el rastreador de mensajes si el tiempo del último mensaje es mayor a 5 segundos
/// - Se utiliza un bucle para limpiar el rastreador de mensajes cada segundo
pub fn message_tracker_cleaner() {
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(1)).await;

            let mut message_tracker = MESSAGE_TRACKER.lock().await;
            if !message_tracker.is_empty() {
                message_tracker.retain(|m| m.last_message_time.elapsed() < Duration::from_secs(5));
            }
        }
    });
}