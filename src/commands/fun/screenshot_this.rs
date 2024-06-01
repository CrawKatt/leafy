use std::fs::remove_file;

use plantita_welcomes::generate_phrase::create_image;
use regex::Regex;
use serenity::all::{ChannelId, CreateMessage, GetMessages, UserId};
use serenity::builder::CreateAttachment;

use crate::utils::{CommandResult, Context};
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapResult};

#[poise::command(
    prefix_command,
    category = "Fun",
    aliases("sst"),
    guild_only,
    track_edits
)]
pub async fn screenshot_this(ctx: Context<'_>, ooc: Option<String>) -> CommandResult {
    let messages = ctx
        .channel_id()
        .messages(&ctx.http(), GetMessages::default())
        .await?;

    let message = messages
        .first()
        .ok_or("No se encontraron mensajes")?;

    let message_replied = &message.referenced_message;
    let Some(message_some) = message_replied else {
        poise::say_reply(ctx, "Debes responder un mensaje para usar este comando").await?;
        return Ok(())
    };

    let content = format!("\"{}\"", &message_some.content);
    let author_id = &message.referenced_message.as_ref().unwrap().author.id; // SAFETY: El `author_id` siempre está disponible en un mensaje referenciado
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let author_avatar = generate_author_avatar(ctx, author_id).await?;
    let author_name = generate_author_name(ctx, author_id).await?;

    // Si se proporciona un canal OOC, se enviará la captura de pantalla a ese canal
    let Some(ooc_channel) = ooc else {
        handle_content(ctx, &content, &content, &author_avatar, &author_name, ctx.channel_id()).await?;
        return Ok(())
    };

    if ooc_channel != "ooc" {
        poise::say_reply(ctx, "El canal proporcionado no es válido").await?;
        return Ok(());
    }

    let ooc_channel = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channels
        .ooc;

    if ooc_channel.is_none() {
        poise::say_reply(ctx, "No se ha establecido un canal OOC").await?;
        return Ok(());
    }

    let ooc_channel = ooc_channel.into_result()?;
    let channel_id = ooc_channel.parse::<ChannelId>()?;

    handle_content(ctx, &content, &content, &author_avatar, &author_name, channel_id).await?;

    Ok(())
}

async fn generate_author_avatar(ctx: Context<'_>, author_id: &UserId) -> UnwrapResult<String> {
    let guild_id = ctx.guild_id().into_result()?; // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let member = guild_id.member(&ctx.http(), author_id).await?;
    let author_avatar = member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado

    Ok(author_avatar)
}

async fn generate_author_name(ctx: Context<'_>, author_id: &UserId) -> UnwrapResult<String> {
    let guild_id = ctx.guild_id().into_result()?; // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let member = guild_id.member(&ctx.http(), author_id).await?;
    let author_name = member.distinct(); // el método distinct devuelve el apodo si existe, de lo contrario, el nombre de usuario
    let author_name = format!("- {author_name}");

    Ok(author_name)
}

/// # Genera y envía la imagen generada al canal
///
/// - Se genera la imagen con el contenido del mensaje referenciado
/// - Se envía la imagen al canal
/// - Se elimina la imagen generada después de enviarla
async fn send_image(
    ctx: Context<'_>,
    channel_id: ChannelId,
    author_avatar: &str,
    quoted_content: &str,
    author_name: &str
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let create_image = create_image(author_avatar, quoted_content, author_name, "assets/PTSerif-Regular.ttf", "assets/PTSerif-Italic.ttf").await?;
    let attachment = CreateAttachment::path(&create_image).await?;
    let message = channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
    let ooc_channel_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channels
        .ooc
        .into_result()?
        .parse::<ChannelId>()?;

    if channel_id == ooc_channel_id {
        message.react(&ctx.http(), '🔺').await?;
        message.react(&ctx.http(), '🔻').await?;
    }

    remove_file(create_image)?;

    Ok(())
}

/// # Maneja el contenido del mensaje referenciado
///
/// - Si el mensaje referenciado contiene una mención a un usuario, se reemplazará la mención por el nombre de usuario
///
/// # Errores
/// - Si el mensaje referenciado es demasiado largo, se enviará un mensaje de error
/// - El mensaje referenciado no puede ser mayor a 72 caracteres
async fn handle_content(
    ctx: Context<'_>,
    content: &str,
    quoted_content: &str,
    author_avatar: &str,
    author_name: &str,
    channel_id: ChannelId
) -> CommandResult {

    let quoted_content = if content.contains("<@") {
        generate_mention(ctx, content, quoted_content).await?
    } else {
        quoted_content.to_string()
    };

    let quoted_content = &*remove_discord_emojis(&quoted_content)?;

    if quoted_content.len() > 72 {
        poise::say_reply(ctx, "El mensaje referenciado es demasiado largo").await?;
        return Ok(());
    }

    send_image(ctx, channel_id, author_avatar, quoted_content, author_name).await?;

    Ok(())
}

/// # Extrae el nombre de usuario de un miembro
///
/// - Esto es necesario para evitar sobrepasar el límite de caracteres en la imagen.
/// - Si el miembro tiene un apodo, se devolverá el apodo, de lo contrario, se devolverá el nombre de usuario
async fn extract_username(ctx: Context<'_>, user_id: &UserId) -> UnwrapResult<String> {
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let member = guild_id.member(&ctx.http(), user_id).await?;
    let author_name = member.distinct();
    let mention = format!("@{author_name}");

    Ok(mention)
}

/// # Extrae y remueve los emojis de Discord en el mensaje
/// 
/// - Esto es necesario para evitar sobrepasar el límite de caracteres en la imagen
/// - Si el mensaje contiene emojis de Discord, se eliminarán del mensaje y se devolverá el mensaje sin emojis
fn remove_discord_emojis(content: &str) -> UnwrapResult<String> {
    let emoji_pattern = Regex::new(r"<a?:.+:\d+>")?;
    let words: Vec<&str> = content.split_whitespace().collect();
    let result: Vec<&str> = words
        .iter()
        .filter(|&word| !emoji_pattern.is_match(word))
        .copied()
        .collect();

    Ok(result.join(" "))
}

/// # Genera una mención a un usuario en el mensaje referenciado
///
/// - Esto es necesario para evitar sobrepasar el límite de caracteres en la imagen
/// ya que las menciones de usuario no son interpretadas como @usuario sino como
/// `<@user_id>`
///
async fn generate_mention(ctx: Context<'_>, content: &str, quoted_content: &str) -> UnwrapResult<String> {
    let user_id = content.split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()?;

    let extracted_username = extract_username(ctx, &user_id).await?;
    let mention = format!("<@{user_id}>");
    let fixed_quoted_content = quoted_content.replace(&mention, &extracted_username);

    Ok(fixed_quoted_content)
}