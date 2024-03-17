use serenity::all::{CreateMessage, GetMessages};
use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::UnwrapLog;
use serenity::builder::CreateAttachment;
use plantita_welcomes::generate_phrase::create_image;

#[poise::command(
    prefix_command,
    category = "Fun",
    aliases("sst"),
    guild_only,
    track_edits
)]
pub async fn screenshot_this(ctx: Context<'_>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages.first().unwrap_log("No se pudo obtener el mensaje", module_path!(), line!())?;
    let content = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.content;
    let author_id = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.id;
    let default_avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.default_avatar_url();
    let avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.avatar_url().unwrap_or_else(|| default_avatar.to_string());
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let author_member = guild_id.member(&ctx.http(), author_id).await?;
    let name = author_member.nick.unwrap_or_else(|| author_member.user.global_name.unwrap_or(author_member.user.name));
    let author_name = format!("- {name}");
    let quoted_content = format!("\"{content}\"");
    let channel_id = ctx.channel_id();
    let font_path = "assets/PTSerif-Regular.ttf";
    let italic_font_path = "assets/PTSerif-Italic.ttf";
    let create_image = create_image(avatar, &quoted_content, &author_name, font_path, italic_font_path).await?;
    let attachment = CreateAttachment::path(&create_image).await?;

    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

    Ok(())
}