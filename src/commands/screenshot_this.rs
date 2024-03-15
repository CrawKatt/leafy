use serenity::all::{CreateMessage, GetMessages};
use crate::utils::{CommandResult, Context, Error};
use crate::utils::misc::debug::UnwrapLog;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use reqwest::get;
use serenity::builder::CreateAttachment;

#[poise::command(
    prefix_command,
    category = "Fun",
    guild_only,
    track_edits
)]
pub async fn screenshot_this(ctx: Context<'_>) -> CommandResult {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let message = messages.first().unwrap_log("No se pudo obtener el mensaje", module_path!(), line!())?;
    let content = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.content;
    let author = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.global_name;
    let default_avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.default_avatar_url();
    let avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.avatar_url().unwrap_or_else(|| default_avatar.to_string());
    let name = author.as_ref().unwrap_log("No se pudo obtener el nombre del autor", module_path!(), line!())?;
    let author_name = format!("- {name}");
    let quoted_content = format!("\"{content}\"");
    let channel_id = ctx.channel_id();
    let create_image = create_image(avatar, &quoted_content, &author_name).await?;
    let attachment = CreateAttachment::path(&create_image).await?;

    channel_id.send_files(&ctx.http(), vec![attachment], CreateMessage::default()).await?;

    Ok(())
}

async fn create_image(avatar: &str, content: &str, name: &str) -> Result<String, Error> {
    // Descarga la imagen del avatar
    let resp = get(avatar).await?;
    let bytes = resp.bytes().await?;
    let mut avatar_img = image::load_from_memory(&bytes)?.to_rgba8();
    // Redimensiona el avatar a un tamaño más pequeño
    avatar_img = image::imageops::resize(&avatar_img, 100, 100, image::imageops::FilterType::Nearest);

    // Crea una nueva imagen con un tamaño específico y fondo negro
    let mut img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_pixel(600, 350, Rgba([0u8, 0u8, 0u8, 255u8]));

    // Dibuja el avatar en la imagen en la posición deseada (un poco más a la derecha y cerca del centro)
    let avatar_x = 80; // 50 pixels to the right
    let avatar_y = (img.height() / 2) - (avatar_img.height() / 2);
    image::imageops::overlay(&mut img, &avatar_img, avatar_x, i64::from(avatar_y));

    // Carga la fuente para el texto del autor
    let font = Vec::from(include_bytes!("../../assets/PTSerif-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap(); // SAFETY: Siempre hay una fuente, en caso de fallo comprobar la ruta

    // Carga la fuente Italic para el texto citado
    let italic_font = Vec::from(include_bytes!("../../assets/PTSerif-Italic.ttf") as &[u8]);
    let italic_font = Font::try_from_vec(italic_font).unwrap(); // SAFETY: Siempre hay una fuente, en caso de fallo comprobar la ruta

    // Dibuja el texto en la imagen en la posición deseada (más a la derecha y a una altura similar a la del avatar)
    let height = 30.0; // Increase the text size
    let scale = Scale { x: height, y: height }; // Make sure x and y are the same to avoid squishing
    let text_x = img.width() - 350; // 350 pixels from the right edge
    let text_y = avatar_y + 25; // Este valor mueve el texto del contenido del mensaje hacia arriba o abajo dependiendo del valor ("+" para abajo, "-" para arriba")

    // Define current_height before drawing the content
    let mut current_height = text_y;

    // Divide el contenido en palabras y dibuja cada línea por separado
    let max_width = 300; // Maximum width of the text
    let words = content.split_whitespace();
    let mut line = String::new();
    for word in words {
        let new_line = format!("{line} {word}");
        let glyphs = italic_font.glyphs_for(new_line.chars());
        let line_width = glyphs.map(|g| g.scaled(scale).h_metrics().advance_width).sum::<f32>();
        if line_width > max_width as f32 {
            // Draw the line and start a new one
            draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), text_x as i32, current_height as i32, scale, &italic_font, &line);
            current_height += height as u32; // Move to the next line
            line = word.to_string();
        } else {
            line = new_line;
        }
    }
    // Draw the last line
    draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), text_x as i32, current_height as i32, scale, &italic_font, &line);

    // Dibuja el nombre del autor en la imagen en una posición independiente del contenido del mensaje
    let name_y = img.height() - 100; // 100 pixels from the bottom edge
    let name_x = img.width() - 300; // 300 pixels from the right edge
    draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), name_x as i32, name_y as i32, scale, &font, name);

    // Guarda la imagen
    let path = format!("/tmp/{content}_phrase.png");
    img.save(&path)?;

    Ok(path)
}