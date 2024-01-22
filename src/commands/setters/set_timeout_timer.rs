use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_timeout_timer;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct SetTimeoutTimer {
    pub time: u64,
    pub guild_id: GuildId,
}

impl SetTimeoutTimer {
    pub const fn new(time: u64, guild_id: GuildId) -> Self {
        Self { time, guild_id }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("time_out_timer")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE time_out_timer SET time = $time";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("time", self.time))
            .await?
            .take(0)?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_timer";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn set_timeout_timer(
    ctx: Context<'_>,
    #[autocomplete = "args_set_timeout_timer"]
    #[description = "The time to set as the time out timer"] set_time: String,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let time_out_timer = match set_time.as_str() {
        "5 Minutos" => SetTimeoutTimer::new(300, ctx.guild_id().unwrap_or_default()),
        "30 Minutos" => SetTimeoutTimer::new(1800, ctx.guild_id().unwrap_or_default()),
        "60 Minutos" => SetTimeoutTimer::new(3600, ctx.guild_id().unwrap_or_default()),
        _ => SetTimeoutTimer::new(60, ctx.guild_id().unwrap_or_default()),
    };

    //let time_out_timer = SetTimeoutTimer::new(tiempo_de_time_out, ctx.guild_id().unwrap_or_default());
    let existing_data = time_out_timer.verify_data().await?;
    let time_out_timer = existing_data.as_ref().map_or(&time_out_timer, |existing_data| existing_data);
    let set_time = time_out_timer.time;

    let Some(_) = existing_data else {
        time_out_timer.save_to_db().await?;
        poise::say_reply(ctx, format!("The time out timer has been set to {set_time} seconds")).await?;

        return Ok(())
    };

    time_out_timer.update_in_db().await?;
    poise::say_reply(ctx, format!("The time out timer has been updated to {set_time} seconds")).await?;

    Ok(())
}