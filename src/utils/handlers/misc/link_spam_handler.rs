use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::Mutex;
use crate::commands::setters::set_to_blacklist::BlackListData;
use serenity::all::{ChannelId, GetMessages, GuildId, Member, Message, UserId};
use poise::serenity_prelude as serenity;
use crate::DB;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::utils::handlers::misc::everyone_case::handle_everyone;

pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").map_or(None, |url_re| url_re.find(text).map(|m| m.as_str().to_string()))
}

static MESSAGE_TRACKER: Lazy<Arc<Mutex<HashMap<UserId, HashMap<String, HashSet<ChannelId>>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

pub async fn handle_blacklist_link(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    link: String,
    data: &MessageData,
    admin_role_id: &Option<String>,
    time: i64
) -> CommandResult {
    let member = &mut guild_id.member(&ctx.http, new_message.author.id).await?;
    let blacklist_link = BlackListData::get_blacklist_link(guild_id, &link).await?;
    if let Some(blacklist_link) = blacklist_link {
        if new_message.content.contains(&blacklist_link) {
            let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
            handle_everyone(admin_role_id.to_owned(), member, ctx, time, new_message).await?;
            return Ok(());
        }
    }

    // Comienza el seguimiento de mensajes
    let message_content = new_message.content.clone();
    let channel_id = new_message.channel_id;

    spam_checker(message_content, channel_id, admin_role_id, member, ctx, time, new_message).await?;

    Ok(())
}

async fn spam_checker(
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
    let user_messages = message_tracker.entry(author_id).or_default();
    let message_channels = user_messages.entry(message_content.clone()).or_default();
    message_channels.insert(channel_id);

    // Guarda los ChannelId de los mensajes para borrarlos más tarde
    let message_channels_to_delete: Vec<ChannelId> = message_channels.iter().copied().collect();

    // Banea al usuario si el límite de mensajes es alcanzado y borra los mensajes
    if message_channels.len() >= 3 {
        handle_everyone(admin_role_id.to_owned(), member, ctx, time, new_message).await?;

        // Borra cada mensaje individualmente
        for channel_id in message_channels_to_delete {
            let channel = channel_id.to_channel(&ctx).await?;
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

        // Limpia completamente el HashMap para reiniciar el rastreo de mensajes
        user_messages.clear();
    }

    // Espera 10 segundos antes de borrar los mensajes en caso de que no sea un link Spam
    tokio::time::sleep(Duration::from_secs(10)).await;

    Ok(())
}