use serenity::all::MessageUpdateEvent;
use poise::serenity_prelude as serenity;
use crate::commands::set_log_channel::GuildData;
use crate::DB;
use crate::events::Error;
use crate::utils::embeds::edit_message_embed;
use crate::utils::MessageData;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> Result<(), Error> {
    if event.author.clone().unwrap().bot {
        println!("Message author is a bot");
        return Ok(());
    }

    let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
    let old_message: Option<MessageData> = DB
        .query(sql_query)
        .bind(("message_id", event.id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(database_message) = old_message else {
        println!("Failed to get database message");
        return Ok(())
    };

    let old_content = database_message.message_content;

    let new_content = event.content.clone().unwrap_or_default();

    if old_content == new_content {
        println!("Message content is the same");
        return Ok(());
    }

    let log_channel_database = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(log_channel_database)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_or_default().log_channel_id;
    println!("Log channel: {}", log_channel);

    let message_content = format!("\n**Antes:** {}\n**Después:** {}", old_content, new_content);

    println!("Message edited: {}", message_content);

    edit_message_embed(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await;

    Ok(())
}