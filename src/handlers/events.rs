use poise::serenity_prelude as serenity;
use serenity::FullEvent;

use crate::DB;
use crate::handlers::{interactions, presence_handler};
use crate::handlers::messages::deleted::delete_message_handler;
use crate::handlers::messages::edited::edited_message_handler;
use crate::handlers::messages::sent::message_handler;
use crate::handlers::misc::reaction_add::vote_react;
use crate::handlers::welcome_event::welcome_handler;
use crate::utils::CommandResult;

/// # Esta función maneja los eventos de Discord
///
/// ## Eventos manejados:
/// - `Ready`: Imprime el nombre del Bot al iniciar sesión
/// - `Message`: Maneja los mensajes enviados en un servidor
/// - `MessageDelete`: Maneja los mensajes eliminados en un servidor
/// - `MessageUpdate`: Maneja los mensajes editados en un servidor
/// - `GuildMemberAddition`: Maneja la llegada de un nuevo miembro a un servidor
/// - `ReactionAdd`: Maneja las reacciones a los mensajes
pub async fn event_handler(ctx: &serenity::Context, event: &FullEvent) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    match event {
        FullEvent::Ready { data_about_bot } => println!("Logged in as {}", data_about_bot.user.name),
        FullEvent::Message { new_message } => message_handler(ctx, new_message).await?,
        FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => delete_message_handler(ctx, channel_id, deleted_message_id).await?,
        FullEvent::MessageUpdate { event, .. } => edited_message_handler(ctx, event).await?,
        FullEvent::GuildMemberAddition { new_member} => welcome_handler(ctx, new_member).await?,
        FullEvent::ReactionAdd { add_reaction } => vote_react(ctx, add_reaction).await?,
        FullEvent::TypingStart { event, .. } => presence_handler::handler(event).await?,
        FullEvent::PresenceUpdate { .. } => (),
        FullEvent::InteractionCreate { interaction, .. } => interactions::handler(ctx, interaction).await?,

        /*
        serenity::FullEvent::PresenceUpdate { .. } => {
            // todo: implement presence update handler
            //println!("Event Presence updated: {:?}", new_data.user);
        }

        serenity::FullEvent::ReactionRemove { .. } => {
            // todo: implement reaction remove handler
            //println!("Event Reaction Remove: {:?}", remove_reaction);
        }
        */

        _ => {
            #[cfg(debug_assertions)] // Macro para imprimir solo en modo Debug
            println!("Unhandled event: {:?}", event.snake_case_name());
        }
    }

    Ok(())
}