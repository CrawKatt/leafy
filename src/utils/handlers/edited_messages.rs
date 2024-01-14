use serenity::all::{ChannelId, GuildId, Message, MessageId, MessageUpdateEvent, UserId};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use crate::DB;
use crate::events::Error;

// Definir la estructura EditedMessageData
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EditedMessageData {
    message_id: MessageId,
    old_content: String,
    new_content: String,
    author_id: UserId,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
}

impl EditedMessageData {
    pub fn new(
        message_id: MessageId,
        old_content: String,
        new_content: String,
        author_id: UserId,
        channel_id: ChannelId,
        guild_id: Option<GuildId>
    ) -> Self {
        Self {
            message_id,
            old_content,
            new_content,
            author_id,
            channel_id,
            guild_id,
        }
    }
}

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent, new: &Option<Message>, old_if_available: &Option<Message>) -> Result<(), Error> {
    if event.author.clone().unwrap().bot {
        println!("Message author is a bot");
        return Ok(());
    }

    let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
    let edited_message: Option<EditedMessageData> = DB
        .query(sql_query)
        .bind(("message_id", event.id)) // pasar el valor
        .await?
        .take(0)?;
    println!("edited_message: {:?}", edited_message);

    // Obtener el mensaje anterior
    let old_message = match old_if_available {
        Some(message) => message,
        None => {
            println!("Failed to get old message");
            return Ok(())
        }
    };

    // Obtener el mensaje nuevo
    let new_message = match new {
        Some(message) => message,
        None => {
            println!("Failed to get new message");
            return Ok(())
        }
    };

    // Si el mensaje anterior es igual al nuevo, no hacer nada
    if old_message.content == new_message.content {
        println!("Message content is the same");
        return Ok(());
    }

    // Crear una instancia de EditedMessageData con los datos del mensaje editado
    let data = EditedMessageData::new(
        new_message.id,
        old_message.content.clone(),
        new_message.content.clone(),
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
    );

    // Guardar el mensaje editado en la base de datos
    let _created: Vec<EditedMessageData> = DB.create("edited_messages").content(data).await?;

    // Imprimir en la consola el contenido anterior y el nuevo del mensaje editado
    println!("Mensaje editado:\nAnterior: {}\nNuevo: {}", old_message.content, new_message.content);

    Ok(())
}