use crate::DB;
use crate::utils::{CommandResult, Context};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WelcomeMessageData {
    pub guild_id: String,
    pub message: String,
}

impl WelcomeMessageData {
    pub fn new(
        guild_id: serenity::GuildId,
        message: String,
    ) -> Self {
        Self {
            guild_id: guild_id.to_string(),
            message,
        }
    }

    /// Guarda el link en la base de datos
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("welcome_message")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn update_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE welcome_message SET message = $message WHERE guild_id = $guild_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("message", self.message.to_string()))
            .bind(("guild_id", self.guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Verifica si el link ya se encuentra en la base de datos
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM welcome_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(existing_data)
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
pub async fn set_welcome_message(
    ctx: Context<'_>,
    #[description = "Mensaje de bienvenida"] message: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let data = WelcomeMessageData::new(guild_id, message);
    let existing_data = data.verify_data().await?;

    if existing_data.is_some() {
        data.update_to_db().await?;
        poise::say_reply(ctx, "Mensaje de bienvenida actualizado").await?;

        return Ok(())
    }

    data.save_to_db().await?;
    poise::say_reply(ctx, "Mensaje de bienvenida guardado").await?;

    Ok(())
}