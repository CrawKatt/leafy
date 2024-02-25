use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::{UnwrapErrors, UnwrapLog};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Joke {
    pub is_active: bool,
    pub guild_id: String,
    pub target: String,
}

impl Joke {
    pub const fn new(guild_id: String, is_active: bool, target: String) -> Self {
        Self {
            is_active,
            guild_id,
            target,
        }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("joke")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM joke WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn switch(&mut self, enable: bool) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE joke SET is_active = $is_active WHERE guild_id = $guild_id";
        DB.query(sql_query)
            .bind(("is_active", enable))
            .bind(("guild_id", self.guild_id.to_string()))
            .await?;

        self.is_active = enable;

        Ok(())
    }

    pub async fn get_joke_target_id(guild_id: GuildId) -> Result<u64, UnwrapErrors> {
        let sql_query = "SELECT * FROM joke WHERE guild_id = $guild_id";
        let database_info: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .await?
            .take(0)?;

        let joke_target_data = database_info.unwrap_log("Failed to get joke target id", module_path!(), line!())?;
        let joke_target_id = joke_target_data.target.parse::<u64>().unwrap_log("Failed to parse joke target id", module_path!(), line!())?;

        Ok(joke_target_id)
    }

    pub async fn get_joke_status(guild_id: GuildId) -> Result<bool, UnwrapErrors> {
        let sql_query = "SELECT * FROM joke WHERE guild_id = $guild_id";
        let database_info: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .await?
            .take(0)?;

        let joke_data = database_info.unwrap_log("Failed to get joke status", module_path!(), line!())?;
        let joke_status = joke_data.is_active;

        Ok(joke_status)
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral
)]
pub async fn joke(
    ctx: Context<'_>,
    #[description = "The user id to set as the joke objetive"] target: UserId,
    #[description = "Enable or disable the joke command"] enable: bool,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_log("Failed to get guild id", module_path!(), line!())?;
    let joke = Joke::new(guild_id.to_string(), enable, target.to_string());
    let existing_data = joke.verify_data().await?;

    let Some(mut existing_data) = existing_data else {
        joke.save_to_db().await?;
        let status = if enable { "habilitado" } else { "deshabilitado" };
        ctx.say(format!("Comando de broma activado\n Status : {status}")).await?;
        return Ok(())
    };

    existing_data.switch(enable).await?;
    println!("Swtich status: {}", existing_data.is_active);
    let status = if existing_data.is_active { "habilitado" } else { "deshabilitado" };
    ctx.say(format!("Joke command {status}")).await?;

    Ok(())
}