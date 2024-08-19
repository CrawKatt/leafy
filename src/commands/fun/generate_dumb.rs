use std::fs::remove_file;

use image::DynamicImage;
use plantita_welcomes::create_welcome::combine_images;
use reqwest::get;
use serenity::all::{CreateMessage, Member, UserId};
use serenity::builder::CreateAttachment;

use crate::commands::fun::get_target_info;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapResult;

#[poise::command(
    prefix_command,
    category = "Fun",
    guild_only,
    user_cooldown = 10,
    track_edits
)]
pub async fn dumb(ctx: Context<'_>, target: Option<Member>) -> CommandResult {
    let (target_id, target_avatar) = get_target_info(&ctx, target).await?;
    let channel_id = ctx.channel_id();
    let mut background = image::open("./assets/dumb_background.png")?;
    let file = generate_dumb(&mut background, target_avatar, &target_id, 800, 80, 512).await?;
    let attachment = CreateAttachment::path(&file).await?;

    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
    remove_file(&file)?;

    Ok(())
}

async fn generate_dumb(
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
    img.resize(512, 512, image::imageops::Lanczos3);

    // Guarda la imagen del avatar en un archivo temporal
    let output_path = format!("/tmp/{target_id}_dumb.png");
    combine_images(background, &img, x, y, avatar_size)?;
    background.save(&output_path)?;

    Ok(output_path)
}