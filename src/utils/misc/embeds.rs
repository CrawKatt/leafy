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
) -> Option<Message> {
    let author_mention = format!("<@{author_id}>");
    let description = format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}");
    let footer = "Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.";
    let embed = create_embed(&author_id.to_user(&ctx.http).await.unwrap_or_default(), &description, footer);
    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.ok()
}

pub async fn edit_message_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &str,
    user: User,
) -> Message {
    let author_mention = format!("<@{author_id}>");
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();

    let embed = CreateEmbed::default()
        .title("Mensaje editado")
        .description(format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {message_content}"))
        .author(CreateEmbedAuthor::new(&author_user.name)
            .name(&author_user.name)
            .icon_url(author_user.avatar_url().as_deref().unwrap_or_default()))
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new("Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona."));

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.unwrap_or_else(|why| {
        println!("Could not send message: {why}");
        Message::default()
    })
}

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
    let embed = create_embed(&author_user, &description, footer);

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
    let embed = create_embed_for_audio(&author_user, &description, footer);

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

fn create_embed(author_user: &User, description: &str, footer: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(description)
        .author(CreateEmbedAuthor::new(&author_user.name)
            .name(&author_user.name)
            .icon_url(author_user.avatar_url().as_deref().unwrap_or_default()))
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new(footer))
}

fn create_embed_for_audio(author_user: &User, description: &str, footer: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title("Audio eliminado")
        .description(description)
        .author(CreateEmbedAuthor::new(&author_user.name)
            .name(&author_user.name)
            .icon_url(author_user.avatar_url().as_deref().unwrap_or_default()))
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new(footer))
}

pub async fn send_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &str,
    user: User,
) -> Message {
    let author_mention = format!("<@{author_id}>");
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);
    let author_user = author_id.to_user(&ctx.http).await.unwrap_or_default();

    let embed = CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(format!("Autor del mensaje: {}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {}", author_mention, &message_content))
        .author(CreateEmbedAuthor::new(&author_user.name)
            .name(&author_user.name)
            .icon_url(author_user.avatar_url().as_deref().unwrap_or_default()))
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new("Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona."));

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, CreateMessage::default())).await.unwrap_or_else(|why| {
        log_handle!("Could not send message: {why}");
        Message::default()
    })
}

fn create_message_embed(embed: CreateEmbed, m: CreateMessage) -> CreateMessage {
    m.embed(embed)
}