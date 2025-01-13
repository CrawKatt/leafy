use crate::utils::config::AutoRole;
use crate::utils::{CommandResult, Context};
use serenity::all::{ChannelId, CreateMessage, EmojiId, ReactionType};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_MESSAGES",
    guild_only,
    track_edits,
    ephemeral
)]
pub async fn set_autorole_message(
    ctx: Context<'_>,
    channel: ChannelId,
    content: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let data = AutoRole::get_assignments(guild_id).await?;
    let Some(data) = data else {
        ctx.say("No se encontraron asignaciones de autoroles para este servidor.").await?;
        return Ok(());
    };

    if data.assignments.is_empty() {
        ctx.say("No hay asignaciones de emojis y roles para reaccionar.").await?;
        return Ok(());
    }

    let builder = CreateMessage::default().content(&content);
    let message = channel.send_message(&ctx.http(), builder).await?;

    for assignment in &data.assignments {
        let emoji_id: EmojiId = assignment.emoji_id.parse()?;
        let reaction = ReactionType::Custom { animated: false, id: emoji_id, name: Some(assignment.emoji_name.clone()) };
        message.react(&ctx.http(), reaction).await?;
    }

    AutoRole::add_message(guild_id, channel.to_string(), message.id.to_string()).await?; // Todo: Fixme

    ctx.say("Mensaje de autorol creado, y reacciones añadidas.").await?;
    Ok(())
}