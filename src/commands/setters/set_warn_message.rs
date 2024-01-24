use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use crate::DB;
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WarnMessageData {
    pub guild_id: GuildId,
    pub warn_message: String,
}

impl WarnMessageData {
    pub const fn new(guild_id: GuildId, warn_message: String) -> Self {
        Self { guild_id, warn_message }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("warn_message")
            .content(self)
            .await?;

        println!("Created warn message: {:?}", self.warn_message);

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warn_message SET warn_message = $warn_message";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warn_message", &self.warn_message))
            .await?
            .take(0)?;

        println!("Updated warn message: {:?}", self.warn_message);

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warn_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        println!("Verified warn message: {:?}", self.warn_message);

        Ok(existing_data)
    }
}

/// Establece el mensaje de advertencia
#[poise::command(prefix_command, slash_command)]
pub async fn set_warn_message(
    ctx: Context<'_>,
    #[description = "The message to set as the warn message"] warn_message: String,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id", line!(), module_path!())?;
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

    let data = WarnMessageData::new(ctx.guild_id().unwrap_log("Could not get the guild_id", line!(), module_path!())?, warn_message.clone());
    let existing_data = data.verify_data().await?;

    if existing_data.is_some() {
        data.update_in_db().await?;
    } else {
        data.save_to_db().await?;
    }

    poise::say_reply(ctx, format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;

    Ok(())
}