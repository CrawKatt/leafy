use std::path::PathBuf;
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::all::{ChannelId, CreateAttachment, CreateEmbedAuthor, CreateMessage, Message, User, UserId};
use crate::log_handle;
use crate::utils::Error;

pub async fn edit_message_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> Option<Message> {
    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.";
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();
    let embed = create_embed_common(&author_user, "Mensaje editado", &description, footer);
    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.ok()
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

/* // todo: Utilizar este embed como un embed para archivos adjuntos
pub async fn send_welcome_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    message_content: String,
    welcome_image: String,
) -> Result<Message, Error> {

    let file_name = welcome_image.split('/').last().unwrap_or_default();
    println!("File name: {:?}", file_name);
    let attachment_image = CreateAttachment::path(welcome_image.clone()).await?;
    let footer = "Por favor, no olvides leer las reglas del servidor";
    let embed = create_embed_welcome("Bienvenido al servidor", &message_content.to_string(), footer, file_name.to_string());
    let message = log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default().add_file(attachment_image))).await?;

    Ok(message)
}
*/

pub async fn send_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> Option<Message> {
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_else(|why| {
        println!("Could not get author user: {why}");
        User::default()
    });

    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.";
    let embed = create_embed_common(&author_user, "Mensaje eliminado", &description, footer);

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.ok()
}

pub async fn send_embed_with_attachment(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    attachment_path: &String,
) -> Option<Message> {
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
    let attachment = CreateAttachment::path(path).await.ok().unwrap_or({
        eprintln!("Could not get attachment");
        CreateAttachment::bytes(Vec::new(), "")
    });

    let message = CreateMessage::default()
        .embed(embed);

    let message_attachment = CreateMessage::default()
        .add_file(attachment);

    log_channel_id.send_message(&ctx.http, message).await.ok();
    log_channel_id.send_message(&ctx.http, message_attachment).await.ok()
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

/* // todo: Utilizar este embed como un embed para archivos adjuntos
fn create_embed_welcome(title: &str, description: &str, footer: &str, welcome_image: String) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .attachment(welcome_image)
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new(footer))
}
*/

pub async fn send_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &str,
    user: User,
) -> Message {
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);
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