use std::path::PathBuf;
use plantita_audio::convert_to_mp3;
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::all::{ChannelId, CreateAttachment, CreateEmbedAuthor, CreateMessage, Message, User, UserId};

// LOS EMBEDS NO NOTIFICAN SI SE MENCIONA CON @ A UN USUARIO
pub async fn edit_message_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> serenity::Result<Message> {
    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: Las menciones a usuarios con @ no mencionan a los usuarios si están dentro de un embed.";
    let author_user = author_id.to_user(&ctx.http).await?;
    let embed = create_embed_common(&author_user, "Mensaje editado", &description, footer);

    log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed)).await
}

// LOS EMBEDS NO NOTIFICAN SI SE MENCIONA CON @ A UN USUARIO
pub async fn send_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> serenity::Result<Message> {
    let author_user = author_id.to_user(&ctx.http).await?;
    let description = format!("Autor del mensaje: <@{author_id}>\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: Las menciones a usuarios con @ no mencionan a los usuarios si están dentro de un embed.";
    let embed = create_embed_common(&author_user, "Mensaje eliminado", &description, footer);

    log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed)).await
}

pub async fn send_embed_with_attachment(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    attachment_path: &str,
) -> serenity::Result<Message> {
    let author_user = author_id.to_user(&ctx.http).await?;
    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>");
    let footer = "Nota: Los audios de Discord no pueden incrustarse dentro de un embed, por lo que el Bot debe enviar dos mensajes al Log.";
    let embed = create_embed_common(&author_user, "Audio eliminado", &description, footer);
    let output_path = "/tmp/converted_audio";

    // Convertir el archivo de audio a mp3
    let mp3_path = convert_to_mp3(attachment_path, output_path)?;

    let path = PathBuf::from(&mp3_path);
    let attachment = CreateAttachment::path(path).await?;

    let message = CreateMessage::default()
        .embed(embed);

    let message_attachment = CreateMessage::default()
        .add_file(attachment);

    log_channel_id.send_message(&ctx.http, message).await?;
    log_channel_id.send_message(&ctx.http, message_attachment).await
}

pub async fn send_warn_embed(
    ctx: &serenity::Context,
    warns: u8,
    tip_image: &str,
    channel_id: ChannelId,
    warn_message: &str,
) -> serenity::Result<Message> {

    let warn_message = format!("{warn_message}\n**Advertencia {warns}/3**");
    let footer = "En el caso de recibir 3 advertencias serás silenciado por una semana. Si estás respondiendo un mensaje considera responder sin el uso de \"@\".";
    let attachment_image = CreateAttachment::path(tip_image).await?;
    let embed = create_warn_embed(&warn_message,&attachment_image, footer);
    let builder = CreateMessage::default()
        .add_file(attachment_image)
        .embed(embed);

    channel_id.send_message(&ctx.http, builder).await
}

fn create_embed_common(author_user: &User, title: &str, description: &str, footer: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .author(CreateEmbedAuthor::new(&author_user.name)
            .name(&author_user.name)
            .icon_url(author_user.avatar_url().unwrap_or_else(|| author_user.default_avatar_url())))
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new(footer))
}

fn create_warn_embed(warn_message: &str, tip_image_mobile: &CreateAttachment, footer: &str) -> CreateEmbed {
    CreateEmbed::default()
        .description(warn_message)
        .attachment(tip_image_mobile.filename.as_str())
        .color(0x00FF_FF00)
        .footer(CreateEmbedFooter::new(footer))
}