use std::fs::remove_file;

use image::imageops::overlay;
use reqwest::get;
use serenity::all::{CreateMessage, Member};
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
pub async fn pride(ctx: Context<'_>, target: Option<Member>) -> CommandResult {
    let (_, target_avatar) = get_target_info(&ctx, target).await?;
    let channel_id = ctx.channel_id();
    let output_path = apply_overlay_to_avatar(&target_avatar, "./assets/pride.png").await?;
    let attachment = CreateAttachment::path(&output_path).await?;

    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
    remove_file(&output_path)?;

    Ok(())
}

async fn apply_overlay_to_avatar(avatar_url: &str, overlay_path: &str) -> UnwrapResult<String> {
    // Descarga la imagen del avatar
    let resp = get(avatar_url).await?;
    let bytes = resp.bytes().await?;
    let avatar_img = image::load_from_memory(&bytes)?.to_rgba8();
    let mut avatar_img = image::imageops::resize(&avatar_img, 256, 256, image::imageops::FilterType::Nearest);
    let overlay_img = image::open(overlay_path)?.to_rgba8();
    overlay(&mut avatar_img, &overlay_img, 0, 0);
    let output_path = "/tmp/avatar_output.png";
    avatar_img.save(output_path)?;

    Ok(output_path.to_string())
}