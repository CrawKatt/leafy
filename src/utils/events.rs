use poise::serenity_prelude as serenity;
use crate::{DB, log_handle};
use crate::utils::CommandResult;
pub use crate::utils::Data;
pub use crate::utils::Error;
use crate::utils::handlers::deleted_messages::delete_message_handler;
use crate::utils::handlers::edited_messages::edited_message_handler;
use crate::utils::handlers::sent_messages::message_handler;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => {
            println!("Event Message: {:?}", new_message.content);
            message_handler(ctx, new_message).await.unwrap_or_else(|why| {
                log_handle!("Could not handle message: {why}");
            });
        }

        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => {
            delete_message_handler(ctx, channel_id, deleted_message_id).await?;
        }

        serenity::FullEvent::MessageUpdate { event, .. } => {
            edited_message_handler(ctx, event).await?;
        }

        serenity::FullEvent::PresenceUpdate { .. } => {
            //println!("Event Presence updated: {:?}", new_data.user);
        }

        _ => println!("Unhandled event: {:?}", event.snake_case_name())
    }

    Ok(())
}