use poise::serenity_prelude as serenity;
use crate::DB;
pub use crate::utils::Data;
pub use crate::utils::Error;
use crate::utils::handlers::deleted_messages::delete_message_handler;
use crate::utils::handlers::message::message_handler;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
) -> Result<(), Error> {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => {
            message_handler(ctx, new_message).await?;
        }

        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => {
            delete_message_handler(ctx, channel_id, deleted_message_id).await?;
        }

        _ => println!("Unhandled event: {:?}", event.snake_case_name())
    }

    Ok(())
}