use poise::serenity_prelude as serenity;
use crate::DB;
use crate::utils::CommandResult;
pub use crate::utils::Data;
pub use crate::utils::Error;
use crate::utils::handlers::deleted_messages::delete_message_handler;
use crate::utils::handlers::edited_messages::edited_message_handler;
use crate::utils::handlers::sent_messages::message_handler;
use crate::utils::handlers::welcome_event::welcome_handler;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => message_handler(ctx, new_message).await?,
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => delete_message_handler(ctx, channel_id, deleted_message_id).await?,
        serenity::FullEvent::MessageUpdate { event, .. } => edited_message_handler(ctx, event).await?,
        serenity::FullEvent::GuildMemberAddition { new_member} => welcome_handler(ctx, new_member).await?,
        serenity::FullEvent::PresenceUpdate { .. } | serenity::FullEvent::TypingStart { .. } => (),

        /*
        serenity::FullEvent::PresenceUpdate { .. } => {
            // todo: implement presence update handler
            //println!("Event Presence updated: {:?}", new_data.user);
        }

        serenity::FullEvent::ReactionAdd { .. } => {
            // todo: implement reaction add handler
            //println!("Event Reaction Add: {:?}", add_reaction);
        }

        serenity::FullEvent::ReactionRemove { .. } => {
            // todo: implement reaction remove handler
            //println!("Event Reaction Remove: {:?}", remove_reaction);
        }
        */

        _ => println!("Unhandled event: {:?}", event.snake_case_name())
    }

    Ok(())
}