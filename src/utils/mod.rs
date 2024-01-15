use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};

pub mod autocomplete;
pub mod embeds;
pub mod handlers;
pub mod events;

pub struct Data {
    pub poise_mentions: String,
    pub client: reqwest::Client,
}

pub type CommandResult = Result<(), Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MessageData {
    pub message_id: MessageId,
    pub message_content: String,
    pub author_id: UserId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

impl MessageData {
    pub const fn new(
        message_id: MessageId,
        message_content: String,
        author_id: UserId,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            message_id,
            message_content,
            author_id,
            channel_id,
            guild_id,
        }
    }
}
