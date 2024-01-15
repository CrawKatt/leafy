use serenity::all::MessageUpdateEvent;
use poise::serenity_prelude as serenity;
use crate::commands::set_log_channel::GuildData;
use crate::DB;
use crate::utils::Error;
use crate::utils::embeds::edit_message_embed;
use crate::utils::MessageData;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> Result<(), Error> {
    if event.author.as_ref().map_or(false, |author| author.bot) {
        return Ok(());
    }

    let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
    let old_message: Option<MessageData> = DB
        .query(sql_query)
        .bind(("message_id", event.id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(database_message) = old_message else {
        return Ok(())
    };

    let old_content = &database_message.message_content;
    let new_content = event.content.as_deref().unwrap_or_default();

    if old_content == new_content {
        return Ok(());
    }

    let log_channel_database = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(log_channel_database)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_or_default().log_channel_id;
    let message_content = format!("\n**Antes:** {old_content}\n**Después:** {new_content}");

    edit_message_embed(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await;

    Ok(())
}