use serde::{Deserialize, Serialize};
use serenity::all::{Channel, ChannelId, GuildId};
use crate::DB;
use crate::utils::autocomplete::args_log_command;
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

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
        let _created: Vec<Self> = DB
            .create("guilds")
            .content(self)
            .await?;

        Ok(())
    }
    async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE guilds SET log_channel_id = $log_channel_id WHERE guild_id = $guild_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("log_channel_id", self.log_channel_id))
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(())
    }
    async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}

/// Establece el canal de logs del servidor
#[poise::command(prefix_command, slash_command)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[autocomplete = "args_log_command"]
    #[description = "The channel to set as the log channel"] channel: Channel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().unwrap_log("Could not get guild_id: `set_log_channel.rs` Line 63")?;
    let author = ctx.author();
    let owner = ctx.guild().unwrap().owner_id;
    let admin_role = AdminData::get_admin_role(guild_id).await?;

    let Some(admin_role) = admin_role else {
        ctx.say("No se ha establecido el rol de administrador").await?;
        return Ok(())
    };

    if !author.has_role(&ctx.serenity_context().http, guild_id, admin_role).await? && author.id != owner {
        ctx.say("No tienes permisos para usar este comando").await?;
        return Ok(())
    }

    let channel_id = channel.id();
    let data = GuildData::new(guild_id, channel_id);
    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        // Si el dato no existe, créalo
        data.save_to_db().await?;
        ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;
        return Ok(());
    };

    // Si el dato ya existe, actualízalo
    data.update_in_db().await?;
    ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;

    Ok(())
}