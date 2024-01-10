use serenity::all::{ChannelId, CreateMessage, GuildId, Message, MessageId};
use serenity::builder::CreateEmbed;
use std::sync::atomic::Ordering;
use poise::serenity_prelude as serenity;
use crate::utils::error::{Data, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("poise") {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Poise has been mentioned {} times", mentions))
                    .await?;
            }
        }

        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            let log_channel = ChannelId::new(1193595925503942688);
            if channel_id == &log_channel {
                return Ok(());
            }

            send_embed(ctx, log_channel, deleted_message_id, guild_id).await;
        }

        _ => {}
    }

    Ok(())
}

async fn send_embed(ctx: &serenity::Context, channel_id: ChannelId, deleted_message_id: &MessageId, guild_id: &Option<GuildId>) -> Option<Message> {
    let embed = CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(format!("Message_id: {deleted_message_id}\nAutor: {}\nChannel_id: {channel_id}", guild_id.unwrap()))
        .color(0x00ff00);

    channel_id.send_message(&ctx.http, create_message_embed(embed, Default::default())).await.ok()
}

fn create_message_embed(embed: CreateEmbed, m: CreateMessage) -> CreateMessage {
    m.embed(embed)
}