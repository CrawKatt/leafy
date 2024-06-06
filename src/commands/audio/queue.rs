use poise::CreateReply;
use serenity::all::CreateEmbed;
use serenity::prelude::TypeMapKey;
use songbird::input::AuxMetadata;

use crate::handlers::misc::buttons::generate_row;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

pub struct AuxMetadataKey;

impl TypeMapKey for AuxMetadataKey {
    type Value = AuxMetadata;
}

pub fn format_metadata(AuxMetadata { title, .. }: &AuxMetadata) -> String {
    title.as_deref().unwrap_or("unknown title").to_string()
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Audio",
    aliases("q"),
)]
pub async fn queue(ctx: Context<'_>) -> CommandResult {
    let songbird = songbird::get(ctx.serenity_context()).await.into_result()?;
    let guild_id = ctx.guild_id().unwrap();
    let call = songbird.get(guild_id).into_result()?;
    let guard = call.lock().await;
    let queue = guard.queue();

    if queue.is_empty() {
        ctx.say("No hay canciones en cola").await?;
        return Ok(());
    }

    if queue.current().is_none() {
        ctx.say("No se obtener la pista actual").await?;
        return Ok(());
    };
    
    let current_queue = queue.current_queue();
    
    let mut description = String::new();
    for (i, track) in current_queue.iter().enumerate() {
        let map = track.typemap().read().await;
        if let Some(metadata) = map.get::<AuxMetadataKey>() {
            description.push_str(&format!("{} - ", i + 1));
            description.push_str(&format_metadata(metadata));
            description.push('\n');
        }
    }
    
    let buttons = generate_row();
    let components = vec![buttons];
    
    let embed = CreateEmbed::default()
        .title("Canciones en cola")
        .description(description)
        .color(0x0000_ff00);

    let builder = CreateReply::default()
        .embed(embed)
        .components(components);
    ctx.send(builder).await?;

    drop(guard);

    Ok(())
}