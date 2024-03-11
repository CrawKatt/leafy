use serenity::all::Message;
use crate::DB;
use crate::utils::{CommandResult, MessageData};

pub async fn attachment_handler(new_message: &Message) -> CommandResult {
    if !new_message.attachments.is_empty() {
        for attachment in new_message.attachments.clone() {
            if attachment.content_type.unwrap_or_default().starts_with("audio") {
                let audio_url = &attachment.url;
                let data = MessageData::new(
                    new_message.id,
                    audio_url,
                    new_message.author.id,
                    new_message.channel_id,
                    new_message.guild_id,
                );

                // Guardar el enlace del archivo de audio en la base de datos
                let _created: Vec<MessageData> = DB.create("audio").content(data).await?;
                println!("Audio file saved to database");
            }
        }
    }

    Ok(())
}