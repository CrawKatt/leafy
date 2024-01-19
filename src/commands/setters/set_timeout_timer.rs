use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::{Context, Error};
use crate::utils::autocomplete::args_set_timeout_timer;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SetTimeoutTimer {
    pub time: u64,
}

impl SetTimeoutTimer {
    pub const fn new(time: u64) -> Self {
        Self { time }
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
    #[description = "The time to set as the time out timer"] time: u64,
) -> Result<(), Error> {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let time_out_timer = SetTimeoutTimer::new(time);
    let existing_data = time_out_timer.verify_data().await?;

    let Some(_) = existing_data else {
        time_out_timer.save_to_db().await?;
        poise::say_reply(ctx, format!("The time out timer has been set to {time} seconds")).await?;

        return Ok(())
    };

    time_out_timer.update_in_db().await?;
    poise::say_reply(ctx, format!("The time out timer has been updated to {time} seconds")).await?;

    Ok(())
}