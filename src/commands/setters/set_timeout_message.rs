use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use crate::DB;
use surrealdb::Result as SurrealResult;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TimeOutMessageData {
    pub guild_id: GuildId,
    pub time_out_message: String,
}

impl TimeOutMessageData {
    pub const fn new(guild_id: GuildId, time_out_message: String) -> Self {
        Self { guild_id, time_out_message }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("time_out_message")
            .content(self)
            .await?;

        println!("Created time out message: {:?}", self.time_out_message);

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE time_out_message SET time_out_message = $time_out_message";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("time_out_message", &self.time_out_message))
            .await?
            .take(0)?;

        println!("Updated time out message: {:?}", self.time_out_message);

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        println!("Verified time out message: {:?}", self.time_out_message);

        Ok(existing_data)
    }
}

/// Establece el mensaje de time out
#[poise::command(prefix_command, slash_command)]
pub async fn set_time_out_message(
    ctx: Context<'_>,
    #[description = "The message to set as the time out message"] time_out_message: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_log("Failed to get guild id: `set_time_out_message` Line 66");
    let time_out_message_data = TimeOutMessageData::new(guild_id, time_out_message.clone());
    let existing_data = time_out_message_data.verify_data().await?;

    let Some(existing_data) = existing_data else {
        time_out_message_data.save_to_db().await?;
        ctx.say(format!("Time out message establecido: {time_out_message}")).await?;
        return Ok(())
    };

    existing_data.update_in_db().await?;
    ctx.say(format!("Time out message actualizado: {time_out_message}")).await?;

    Ok(())
}