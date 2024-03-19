use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId};
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;
use crate::DB;

#[derive(Serialize, Deserialize)]
pub struct OocChannel {
    pub channel_id: String,
    pub guild_id: String
}

impl OocChannel {
    pub fn new(channel_id: ChannelId, guild_id: GuildId) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            guild_id: guild_id.to_string()
        }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("ooc_channel")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        let sql_query = "SELECT * FROM ooc_channel";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        let sql_query = "UPDATE ooc_channel SET channel_id = $channel_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("channel_id", &self.channel_id))
            .await?
            .take(0)?;

        Ok(())
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_ooc_channel(ctx: Context<'_>, ooc_channel: ChannelId) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let data = OocChannel::new(ooc_channel, guild_id);
    let existing_data = data.verify_data().await?;

    if existing_data.is_none() {
        data.save_to_db().await?;
        ctx.say(format!("Canal de Fuera de Contexto establecido en: <#{ooc_channel}>")).await?;
        return Ok(())
    }

    data.update_in_db().await?;

    let message = format!("Canal de Fuera de Contexto establecido en: <#{ooc_channel}>");
    ctx.say(message).await?;

    Ok(())
}