use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId};
use crate::DB;
use crate::utils::debug::UnwrapLog;
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;

#[derive(Serialize, Deserialize, Debug)]
pub struct JokeChannelData {
    pub channel_id: String,
    pub guild_id: String,
}

impl JokeChannelData {
    pub fn new(channel_id: ChannelId, guild_id: GuildId) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            guild_id: guild_id.to_string(),
        }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("joke_channel")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        let sql_query = "SELECT * FROM joke_channel WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}

/// Establece el canal de broma donde se ejecutará la broma a Meica
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_joke_channel(
    ctx: Context<'_>,
    #[description = "The channel to set as the joke channel"] channel: serenity::all::GuildChannel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild id", module_path!(), line!())?;
    let data = JokeChannelData::new(channel.id, guild_id);
    let existing_data = data.verify_data().await?;

    if existing_data.is_none() {
        data.save_to_db().await?;
        ctx.say(format!("Se ha establecido el canal de broma en: <#{}>", channel.id)).await?;
    } else {
        ctx.say("Ya hay un canal de chistes establecido").await?;
    }

    Ok(())
}