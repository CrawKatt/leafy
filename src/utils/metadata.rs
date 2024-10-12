use serde::Deserialize;
use std::fs;
use std::path::Path;
use serenity::all::{CreateEmbed, CreateMessage};
use serenity::builder::CreateEmbedAuthor;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapResult;

#[derive(Deserialize, Debug)]
pub struct VideoMetadata {
    pub title: String,
    pub thumbnail: Option<String>,
}

pub fn read_metadata<P: AsRef<Path>>(path: P) -> UnwrapResult<VideoMetadata> {
    let file_content = fs::read_to_string(path)?;
    let metadata: VideoMetadata = serde_json::from_str(&file_content).unwrap();
    Ok(metadata)
}

pub async fn build_embed(ctx: &Context<'_>, json_path: &str, author_name: &str, author_face: &str) -> CommandResult {
    let metadata = read_metadata(json_path)?;
    
    let title = metadata.title;
    let description = format!("**Solicitado por:** {author_name}");
    let thumbnail = metadata.thumbnail.unwrap_or("".to_string());
    
    let embed = CreateEmbed::new()
        .title(title)
        .author(CreateEmbedAuthor::new(author_name).icon_url(author_face))
        .description(description)
        .thumbnail(thumbnail)
        .color(0x00ff_0000);
    
    let builder = CreateMessage::new().embed(embed);
    ctx.channel_id().send_message(ctx.http(), builder).await?;
    
    Ok(())
}