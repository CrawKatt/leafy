use crate::DB;
use crate::utils::{Context, Error};
use crate::commands::setters::set_timeout_timer::SetTimeoutTimer;

#[poise::command(prefix_command, slash_command)]
pub async fn get_timeout_timer(
    ctx: Context<'_>,
) -> Result<(), Error> {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let sql_query = "SELECT * FROM time_out_timer";
    let time_out_timer: Option<SetTimeoutTimer> = DB
        .query(sql_query)
        .await?
        .take(0)?;

    let time = time_out_timer.unwrap_or_default().time;

    poise::say_reply(ctx, format!("The time out timer is set to {time} seconds")).await?;

    Ok(())
}