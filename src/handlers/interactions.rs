use poise::serenity_prelude as serenity;
use serenity::all::{Context, Interaction};

use crate::handlers::misc::buttons::{ButtonAction, handle_action, handle_and_update};
use crate::utils::CommandResult;
use crate::utils::debug::IntoUnwrapResult;

pub async fn handler(ctx: &Context, interaction: &Interaction) -> CommandResult {
    let Some(mc) = interaction.as_message_component() else { return Ok(()) };
    let guild_id = mc.guild_id.into_result()?;
    let custom_id = mc.data.custom_id.as_str();
    
    #[cfg(debug_assertions)]
    println!("Button pressed: {custom_id}");

    match ButtonAction::from(custom_id) {
        ButtonAction::Skip => handle_action(ctx, guild_id, mc, "Se ha saltado la canción", |queue| queue.skip()).await?,
        ButtonAction::Pause => handle_and_update(ctx, guild_id, mc, "Se ha pausado la canción", |queue| queue.pause()).await?,
        ButtonAction::Resume => handle_and_update(ctx, guild_id, mc, "Se ha reanudado la canción", |queue| queue.resume()).await?,
        ButtonAction::Stop => handle_action(ctx, guild_id, mc, "Se ha detenido la canción", |queue| { queue.stop(); Ok(()) }).await?,
        ButtonAction::Unknown => {
            #[cfg(debug_assertions)]
            println!("Unhandled button: {custom_id}");
        }
    }

    Ok(())
}