use std::collections::HashMap;
use std::fs::remove_file;

use image::DynamicImage;
use plantita_welcomes::create_welcome::combine_images;
use poise::serenity_prelude as serenity;
use reqwest::get;
use serenity::all::{ChannelId, CreateAttachment, User};

use crate::utils::config::load_data;
use crate::utils::debug::UnwrapErrors;
use crate::utils::CommandResult;

pub async fn handler(
    ctx: &serenity::Context,
    new_member: &serenity::Member,
) -> CommandResult {
    let guild_id = new_member.guild_id;
    let user = &new_member.user;
    let channel_id = load_data().channels.welcome.parse::<ChannelId>()?;
    let welcome_message = load_data().messages.welcome;

    let mut background = image::open("assets/background.png")?;
    let file = get_welcome_attachment(&mut background, user, 74, 74, 372).await?;

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("Bienvenido {user} a {}. \n{welcome_message}", guild_id.name(&ctx.cache).unwrap_or_default()));
    let http = &ctx.http;
    let attachment = CreateAttachment::path(&file).await?;

    // En el attachment se puede pasar un archivo de imagen para la bienvenida
    http.send_message(channel_id, vec![attachment], &message_map).await?;

    // Borrar la imágen generada después de usarla
    remove_file(file)?;

    Ok(())
}

async fn get_welcome_attachment(background: &mut DynamicImage, user: &User, x: u32, y: u32, avatar_size: u32) -> Result<String, UnwrapErrors> {
    // Obtén la URL del avatar del usuario
    let avatar_url = user.face();

    // Descarga la imagen del avatar
    let response = get(avatar_url).await?;
    let bytes= response.bytes().await?;

    // Carga la imagen del avatar en memoria y redimensiona a 256x256
    let img = image::load_from_memory(&bytes)?;
    img.resize(256, 256, image::imageops::Lanczos3);

    // Guarda la imagen del avatar en un archivo temporal
    let output_path = format!("/tmp/{}_welcome.png", user.id);
    combine_images(background, &img, x, y, avatar_size)?;
    background.save(&output_path)?;

    Ok(output_path)
}