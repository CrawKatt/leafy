use serenity::all::{ChannelId, CreateMessage, GetMessages};
use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::{UnwrapLog, UnwrapResult};
use serenity::builder::CreateAttachment;
use crate::commands::setters::set_ooc_channel::OocChannel;
use plantita_welcomes::generate_phrase::create_image;
use std::fs::remove_file;

#[poise::command(
    prefix_command,
    category = "Fun",
    aliases("sst"),
    guild_only,
    track_edits
)]
pub async fn screenshot_this(ctx: Context<'_>, ooc: Option<String>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages.first().unwrap_log("No se pudo obtener el mensaje", module_path!(), line!())?;
    let content = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.content;

    if content.len() > 72 {
        poise::say_reply(ctx, "El mensaje referenciado es demasiado largo").await?;
        return Ok(());
    }

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

    // Si se proporciona un canal OOC, se enviará la captura de pantalla a ese canal
    if ooc.is_some() {
        if ooc != Some(String::from("ooc")) {
            poise::say_reply(ctx, "El canal proporcionado no es válido").await?;
            return Ok(());
        }

        let sql_query = "SELECT * FROM ooc_channel WHERE guild_id = $guild_id";
        let existing_data: Option<OocChannel> = crate::DB
            .query(sql_query)
            .bind(("guild_id", &guild_id.to_string()))
            .await?
            .take(0)?;

        if existing_data.is_none() {
            poise::say_reply(ctx, "No se ha establecido un canal OOC").await?;
            return Ok(());
        }

        let ooc_channel = existing_data.unwrap_log("No se pudo obtener el canal OOC", module_path!(), line!())?;
        let channel_u64 = ooc_channel.channel_id.parse::<u64>()?;
        let channel_id = ChannelId::new(channel_u64);

        if content.contains("<@") {

            let fixed_quoted_content = generate_mention(ctx, content, quoted_content).await?;
            let create_image = create_image(avatar, &fixed_quoted_content, &author_name, font_path, italic_font_path).await?;
            let attachment = CreateAttachment::path(&create_image).await?;

            channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

            return Ok(());
        }

        let create_image = create_image(avatar, &quoted_content, &author_name, font_path, italic_font_path).await?;
        let attachment = CreateAttachment::path(&create_image).await?;

        channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
        remove_file(create_image)?;

        return Ok(());
    }

    if content.contains("<@") {

        let fixed_quoted_content = generate_mention(ctx, content, quoted_content).await?;
        let create_image = create_image(avatar, &fixed_quoted_content, &author_name, font_path, italic_font_path).await?;
        let attachment = CreateAttachment::path(&create_image).await?;

        channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

        return Ok(());
    }

    let create_image = create_image(avatar, &quoted_content, &author_name, font_path, italic_font_path).await?;
    let attachment = CreateAttachment::path(&create_image).await?;

    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
    remove_file(create_image)?;

    Ok(())
}

async fn extract_username(ctx: Context<'_>, user_id: &serenity::model::id::UserId) -> UnwrapResult<String> {
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let member = guild_id.member(&ctx.http(), user_id).await?;
    let author_name = member.nick.unwrap_or_else(|| member.user.global_name.unwrap_or(member.user.name));
    let mention = format!("@{author_name}");

    Ok(mention)
}

async fn generate_mention(ctx: Context<'_>, content: &str, quoted_content: String) -> UnwrapResult<String> {
    let user_id = content.split("<@")
        .collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    let user_id = serenity::model::id::UserId::from(user_id);
    let extracted_username = extract_username(ctx, &user_id).await?;
    let mention = format!("<@{user_id}>");
    let fixed_quoted_content = quoted_content.replace(&mention, &extracted_username);

    Ok(fixed_quoted_content)
}