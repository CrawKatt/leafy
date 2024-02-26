use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::misc::debug::{UnwrapLog, UnwrapResult};
use serde::{Serialize, Deserialize};
use serenity::all::{ChannelId, GuildId};
use crate::utils::{CommandResult, Context};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WelcomeChannelData {
    pub guild_id: String,
    pub channel_id: String,
}

impl WelcomeChannelData {
    pub fn new(
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            guild_id: guild_id.to_string(),
            channel_id: channel_id.to_string(),
        }
    }

    /// Guarda el link en la base de datos
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("welcome_channel")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn update_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        //let sql_query = "UPDATE welcome_channel SET channel_id = $channel_id WHERE guild_id = $guild_id";
        let sql_query = "UPDATE welcome_channel SET channel_id = $channel_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("channel_id", self.channel_id.to_string()))
            .bind(("guild_id", self.guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Verifica si el link ya se encuentra en la base de datos
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM welcome_channel WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn get_welcome_channel(guild_id: GuildId) -> UnwrapResult<String> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM welcome_channel WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .await?
            .take(0)?;

        let result = existing_data.unwrap_log("No se encontró el canal de bienvenida", file!(), line!())?.channel_id;

        Ok(result)
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
pub async fn set_welcome_channel(
    ctx: Context<'_>,
    #[description = "Canal de bienvenida"] channel: ChannelId,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_log("Could not get guild id", file!(), line!())?;
    let welcome_channel = WelcomeChannelData::new(guild_id, channel);
    let data = welcome_channel.verify_data().await?;

    if data.is_some() {
        welcome_channel.update_to_db().await?;
        poise::say_reply(ctx, format!("Canal de bienvenida actualizado en <#{channel}>")).await?;

        return Ok(())
    }

    welcome_channel.save_to_db().await?;
    poise::say_reply(ctx, format!("Canal de bienvenida establecido en <#{channel}>")).await?;

    Ok(())
}