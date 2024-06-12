use poise::{Command, serenity_prelude as serenity};
use serenity::all::{ComponentInteraction, ComponentInteractionDataKind, Context, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction};

use crate::commands::info::help::{filter_categories, FOOTER_URL};
use crate::debug;
use crate::handlers::misc::buttons::{ButtonAction, handle_action, handle_and_update};
use crate::utils::{CommandResult, Data, Error};
use crate::utils::debug::IntoUnwrapResult;

/// # Esta función maneja las interacciones de botones
/// - `mc`: La interacción de componente
/// - `ButtonAction`: Enumeración de acciones de botones
/// - `help_action()`: Edita el menú de ayuda con los comandos del Bot filtrados por categoría
pub async fn handler(
    ctx: &Context,
    interaction: &Interaction,
    commands: &[Command<Data, Error>]
) -> CommandResult {
    let Some(mc) = interaction.as_message_component() else { return Ok(()) };
    let guild_id = mc.guild_id.into_result()?;
    let custom_id = mc.data.custom_id.as_str();
    debug!("Button pressed: {custom_id}");

    match ButtonAction::from(custom_id) {
        ButtonAction::HelpMenu => help_action(ctx, mc, commands).await?,
        ButtonAction::Skip => handle_action(ctx, guild_id, mc, "Se ha saltado la canción", |queue| queue.skip()).await?,
        ButtonAction::Pause => handle_and_update(ctx, guild_id, mc, "Se ha pausado la canción", |queue| queue.pause(),true).await?,
        ButtonAction::Resume => handle_and_update(ctx, guild_id, mc, "Se ha reanudado la canción", |queue| queue.resume(), false).await?,
        ButtonAction::Stop => handle_action(ctx, guild_id, mc, "Se ha detenido la canción", |queue| { queue.stop(); Ok(()) }).await?,
        ButtonAction::Unknown => { debug!("Unhandled button: {custom_id}") }
    }

    Ok(())
}

/// # Esta función maneja la acción de un botón de menú de ayuda
/// - `kind`: La categoría de comandos a mostrar
/// - `mc`: La interacción de componente
/// - `filter_categories()`: Filtra los comandos por categoría
pub async fn help_action(
    ctx: &Context,
    mc: &ComponentInteraction,
    commands: &[Command<Data, Error>]
) -> CommandResult {
    let kind = match &mc.data.kind {
        ComponentInteractionDataKind::StringSelect {
            values,
        } => &values[0],
        _ => return Ok(()),
    };
    
    let description = filter_categories(commands, kind);
    let embed = CreateEmbed::default()
        .title("Comandos de Plantita Ayudante")
        .color(0x0000_ff00)
        .footer(CreateEmbedFooter::new("© CrawKatt").icon_url(FOOTER_URL))
        .description(description);
    
    let builder = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);
    
    let response = CreateInteractionResponse::UpdateMessage(builder);
    mc.create_response(&ctx, response).await?;

    Ok(())
}