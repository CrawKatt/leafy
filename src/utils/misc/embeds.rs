use std::path::PathBuf;
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::all::{ChannelId, CreateAttachment, CreateEmbedAuthor, CreateMessage, Message, User, UserId};
use crate::log_handle;

pub async fn edit_message_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> serenity::Result<Message> {
    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.";
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();
    let embed = create_embed_common(&author_user, "Mensaje editado", &description, footer);
    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await
}

pub async fn edit_message_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &str,
    user: User,
) -> serenity::Result<Message> {
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();
    let description = format!("Autor del mensaje: <@{author_id}>\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");

    let embed = create_embed_common(&author_user, "Mensaje editado", &description, "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.");

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await
}

pub async fn send_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> serenity::Result<Message> {
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_else(|why| {
        println!("Could not get author user: {why}");
        User::default()
    });

    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.";
    let embed = create_embed_common(&author_user, "Mensaje eliminado", &description, footer);

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await
}

pub async fn send_embed_with_attachment(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    attachment_path: &String,
) -> serenity::Result<Message> {
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_else(|why| {
        println!("Could not get author user: {why}");
        User::default()
    });

    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>");
    let footer = "Nota: Los audios de Discord no pueden enviarse dentro de un embed y no pueden enviarse como audios reproducibles, por lo que aparecen como dos mensajes y el audio debe descargarse.";
    let embed = create_embed_common(&author_user, "Audio eliminado", &description, footer);

    println!("Attachment path: {attachment_path}");

    let path = PathBuf::from(attachment_path);
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
    let message = CreateMessage::default().add_file(attachment_image);

    channel_id.send_message(&ctx.http, create_message_embed(embed, message)).await
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

// LOS EMBEDS NO NOTIFICAN SI SE MENCIONA CON @
pub async fn send_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &str,
) -> Message {
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();
    let description = format!("Autor del mensaje: <@{author_id}>\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");

    let embed = create_embed_common(&author_user, "Mensaje eliminado", &description, "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.");

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.unwrap_or_else(|why| {
        log_handle!("Could not send message: {}", why);
        Message::default()
    })
}

fn create_message_embed(embed: CreateEmbed, m: CreateMessage) -> CreateMessage {
    m.embed(embed)
}