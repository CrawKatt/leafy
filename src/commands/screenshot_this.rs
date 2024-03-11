use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateMessage, GetMessages};
use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::UnwrapLog;

#[poise::command(
    prefix_command,
    category = "Fun",
    guild_only,
    track_edits
)]
pub async fn screenshot_this(ctx: Context<'_>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages.first().unwrap_log("No se pudo obtener el mensaje", module_path!(), line!())?;
    let content = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.content;
    let author = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.name;
    let default_avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.default_avatar_url();
    let avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.avatar_url().unwrap_or_else(|| default_avatar.to_string());

    let embed = CreateEmbed::default()
        .description(format!("*{content}*"))
        .author(CreateEmbedAuthor::new(author)
            .name(author)
            .icon_url(avatar));

    let channel_id = ctx.channel_id();
    let message = CreateMessage::default().embed(embed);
    channel_id.send_message(&ctx.http(), message).await?;

    Ok(())
}