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
    let author_id = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.id;
    let default_avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.default_avatar_url();
    let avatar = &message.referenced_message.as_ref().unwrap_log("No se pudo obtener el mensaje referenciado", module_path!(), line!())?.author.avatar_url().unwrap_or_else(|| default_avatar.to_string());
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando
    let author_member = guild_id.member(&ctx.http(), author_id).await?;
    let name = author_member.nick.unwrap_or_else(|| author_member.user.global_name.unwrap_or(author_member.user.name));
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
    avatar_img = image::imageops::resize(&avatar_img, 150, 150, image::imageops::FilterType::Nearest);

    // Crea una nueva imagen con un tamaño específico y fondo negro
    let mut img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_pixel(800, 300, Rgba([0u8, 0u8, 0u8, 255u8]));

    // Dibuja el avatar en la imagen en la posición deseada (un poco más a la derecha y cerca del centro)
    let avatar_x = 150; // 50 pixels to the right
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
    let int_height = 30; // Increase the text size
    let scale = Scale { x: height, y: height }; // Make sure x and y are the same to avoid squishing
    let text_x = img.width() - 350; // 350 pixels from the right edge
    let text_x : i32 = text_x.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32
    let text_y = avatar_y + 25; // Este valor mueve el texto del contenido del mensaje hacia arriba o abajo dependiendo del valor ("+" para abajo, "-" para arriba")

    // Define current_height before drawing the content
    let current_height = text_y;
    let mut current_height = current_height.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32

    // Divide el contenido en palabras y dibuja cada línea por separado
    let max_width = 300.0; // Maximum width of the text
    let words = content.split_whitespace();
    let mut line = String::new();
    for word in words {
        let glyphs = italic_font.glyphs_for(word.chars());
        let word_width = glyphs.map(|g| g.scaled(scale).h_metrics().advance_width).sum::<f32>();
        if word_width > max_width {
            // If the word is too long, split it into multiple lines
            let chars = word.chars().collect::<Vec<char>>();
            let mut sub_word = String::new();
            for ch in chars {
                let new_sub_word = format!("{sub_word}{ch}");
                let sub_word_width = italic_font.glyphs_for(new_sub_word.chars()).map(|g| g.scaled(scale).h_metrics().advance_width).sum::<f32>();
                if sub_word_width > max_width {
                    // Draw the line and start a new one
                    draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), text_x, current_height, scale, &italic_font, &sub_word);
                    current_height += int_height; // Move to the next line
                    sub_word = ch.to_string();
                } else {
                    sub_word = new_sub_word;
                }
            }
            line = sub_word;
        } else {
            let new_line = format!("{line} {word}");
            let line_width = italic_font.glyphs_for(new_line.chars()).map(|g| g.scaled(scale).h_metrics().advance_width).sum::<f32>();
            if line_width > max_width {
                // Draw the line and start a new one
                draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), text_x, current_height, scale, &italic_font, &line);
                current_height += int_height; // Move to the next line
                line = word.to_string();
            } else {
                line = new_line;
            }
        }
    }
    // Draw the last line
    draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), text_x, current_height, scale, &italic_font, &line);

    // Dibuja el nombre del autor en la imagen en una posición independiente del contenido del mensaje
    let name_y = img.height() - 100; // 100 pixels from the bottom edge
    let name_y = name_y.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32

    let name_x = img.width() - 300; // 300 pixels from the right edge
    let name_x = name_x.try_into().unwrap_or(i32::MAX); // SAFETY: Si el valor es mayor a i32, se asigna el valor máximo de i32
    draw_text_mut(&mut img, Rgba([255u8, 255u8, 255u8, 255u8]), name_x, name_y, scale, &font, name);

    // Guarda la imagen
    let path = format!("/tmp/{content}_phrase.png");
    img.save(&path)?;

    Ok(path)
}