use poise::{FrameworkContext, serenity_prelude as serenity};
use serenity::FullEvent;

use crate::{DB, debug};
use crate::handlers::{interactions, typing_start, welcome};
use crate::handlers::messages::{deleted, edited, sent};
use crate::handlers::misc::reaction_add::vote_react;
use crate::utils::{CommandResult, Data, Error};

/// # Esta función maneja los eventos de Discord
///
/// ## Eventos manejados:
/// - `Ready`: Imprime el nombre del Bot al iniciar sesión
/// - `Message`: Maneja los mensajes enviados en un servidor
/// - `MessageDelete`: Maneja los mensajes eliminados en un servidor
/// - `MessageUpdate`: Maneja los mensajes editados en un servidor
/// - `GuildMemberAddition`: Maneja la llegada de un nuevo miembro a un servidor
/// - `ReactionAdd`: Maneja las reacciones a los mensajes
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    match event {
        FullEvent::Ready { data_about_bot } => println!("Logged in as {}", data_about_bot.user.name),
        FullEvent::Message { new_message } => sent::handler(ctx, new_message).await?,
        FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => deleted::handler(ctx, channel_id, deleted_message_id).await?,
        FullEvent::MessageUpdate { event, .. } => edited::handler(ctx, event).await?,
        FullEvent::GuildMemberAddition { new_member} => welcome::handler(ctx, new_member).await?,
        FullEvent::ReactionAdd { add_reaction } => vote_react(ctx, add_reaction).await?,
        FullEvent::TypingStart { event } => typing_start::handler(event).await?,
        FullEvent::InteractionCreate { interaction } => interactions::handler(ctx, interaction, &framework).await?,

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

        _ => { debug!("Unhandled event: {:?}", event.snake_case_name()) }
    }

    Ok(())
}