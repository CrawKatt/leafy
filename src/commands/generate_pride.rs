use serenity::all::{CreateMessage, GetMessages, Member};
use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::{UnwrapLog, UnwrapResult};
use serenity::builder::CreateAttachment;
use std::fs::remove_file;
use image::imageops::overlay;
use reqwest::get;

#[poise::command(
    prefix_command,
    category = "Fun",
    aliases("sst"),
    guild_only,
    track_edits
)]
pub async fn pride(ctx: Context<'_>, target: Option<Member>) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando

    if target.is_some() {
        let target_member = target.unwrap_log("No se pudo obtener el miembro", module_path!(), line!())?;
        let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
        let channel_id = ctx.channel_id();
        let output_path = apply_overlay_to_avatar(&target_avatar, "./assets/pride.png").await?;
        let attachment = CreateAttachment::path(&output_path).await?;

        channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;
        remove_file(output_path)?;

        return Ok(())
    }

    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages
        .first()
        .unwrap_log("No se pudo obtener el mensaje", module_path!(), line!())?;

    let target_id = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.id;
    let target_member = guild_id.member(&ctx.http(), target_id).await?;
    let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
    let channel_id = ctx.channel_id();
    let output_path = apply_overlay_to_avatar(&target_avatar, "./assets/pride.png").await?;

    let attachment = CreateAttachment::path(&output_path).await?;
    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

    remove_file(output_path)?;

    Ok(())
}

async fn apply_overlay_to_avatar(avatar_url: &str, overlay_path: &str) -> UnwrapResult<String> {
    // Descarga la imagen del avatar
    let resp = get(avatar_url).await?;
    let bytes = resp.bytes().await?;
    let avatar_img = image::load_from_memory(&bytes)?.to_rgba8();

    // Redimensiona el avatar a 256x256
    let mut avatar_img = image::imageops::resize(&avatar_img, 256, 256, image::imageops::FilterType::Nearest);

    // Abre la imagen semi-transparente
    let overlay_img = image::open(overlay_path)?.to_rgba8();

    // Dibuja la imagen semi-transparente en el avatar
    overlay(&mut avatar_img, &overlay_img, 0, 0);

    // Guarda la imagen resultante
    let output_path = "/tmp/avatar_output.png";
    avatar_img.save(output_path)?;

    Ok(output_path.to_string())
}