use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

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

        Ok(())
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
        ctx.say("Joke command enabled").await?;
        return Ok(())
    };

    existing_data.switch(enable).await?;
    println!("Swtich status: {}", existing_data.is_active);
    let status = if existing_data.is_active { "enabled" } else { "disabled" };
    ctx.say(format!("Joke command {status}")).await?;

    Ok(())
}