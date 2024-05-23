use std::fs::remove_file;

use image::DynamicImage;
use plantita_welcomes::create_welcome::combine_images;
use reqwest::get;
use serenity::all::{CreateMessage, GetMessages, Member, UserId};
use serenity::builder::CreateAttachment;

use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::{IntoUnwrapResult, UnwrapResult};

#[poise::command(
    prefix_command,
    category = "Fun",
    guild_only,
    track_edits
)]
pub async fn furry(ctx: Context<'_>, target: Option<Member>) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando

    if target.is_some() {
        let target_member = target.into_result()?;
        let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
        let target_id = &target_member.user.id;
        let channel_id = ctx.channel_id();
        let mut background = image::open("./assets/furry_backgorund.jpg")?;
        let file = generate_furry(&mut background, target_avatar, target_id, 550, 280, 250).await?;
        let attachment = CreateAttachment::path(&file).await?;

        channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
        remove_file(file)?;

        return Ok(())
    }

    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages
        .first()
        .into_result()?;

    let target_id = &message.referenced_message.as_ref().into_result()?.author.id;
    let target_member = guild_id.member(&ctx.http(), target_id).await?;
    let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
    let channel_id = ctx.channel_id();

    let mut background = image::open("./assets/furry_backgorund.jpg").unwrap();
    let file = generate_furry(&mut background, target_avatar, target_id, 550, 280, 250).await?;

    let attachment = CreateAttachment::path(&file).await?;
    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

    remove_file(file)?;

    Ok(())
}

async fn generate_furry(
    background: &mut DynamicImage,
    target_avatar_url: String,
    target_id: &UserId,
    x: u32,
    y: u32,
    avatar_size: u32
) -> UnwrapResult<String> {
    // Descarga la imagen del avatar
    let response = get(target_avatar_url).await?;
    let bytes= response.bytes().await?;

    // Carga la imagen del avatar en memoria y redimensiona a 256x256
    let img = image::load_from_memory(&bytes)?;
    img.resize(256, 256, image::imageops::Lanczos3);

    // Guarda la imagen del avatar en un archivo temporal
    let output_path = format!("/tmp/{target_id}_furry.jpg");
    combine_images(background, &img, x, y, avatar_size)?;
    background.save(&output_path)?;

    Ok(output_path)
}