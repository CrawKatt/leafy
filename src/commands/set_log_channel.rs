use serde::{Deserialize, Serialize};
use serenity::all::{Channel, ChannelId, GuildId};
use crate::DB;
use crate::utils::autocomplete::autocomplete_log_command;
use surrealdb::Result as SurrealResult;
use crate::utils::{CommandResult, Context};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GuildData {
    pub guild_id: GuildId,
    pub log_channel_id: ChannelId,
}

impl GuildData {
    pub const fn new(guild_id: GuildId, log_channel_id: ChannelId) -> Self {
        Self { guild_id, log_channel_id }
    }
    async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<GuildData> = DB
            .create("guilds")
            .content(self)
            .await?;

        Ok(())
    }
    async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE guilds SET log_channel_id = $log_channel_id WHERE guild_id = $guild_id";
        let _updated: Vec<GuildData> = DB
            .query(sql_query)
            .bind(("log_channel_id", self.log_channel_id))
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(())
    }
    async fn verify_data(&self) -> SurrealResult<Option<GuildData>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
        let existing_data: Option<GuildData> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_log_command"]
    #[description = "The channel to set as the log channel"] channel: Channel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let channel_id = channel.id();
    let data = GuildData::new(ctx.guild_id().unwrap_or_default(), channel_id);
    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        // Si el dato no existe, créalo
        data.save_to_db().await?;
        ctx.say(format!("Log channel establecido: <#{}>", channel_id)).await?;
        return Ok(());
    };

    // Si el dato ya existe, actualízalo
    data.update_in_db().await?;
    ctx.say(format!("Log channel establecido: <#{}>", channel_id)).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn get_log_channel(
    ctx: Context<'_>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel_id = database_info.unwrap_or_default().log_channel_id;
    ctx.say(format!("Log channel is <#{}>", log_channel_id)).await?;

    Ok(())
}