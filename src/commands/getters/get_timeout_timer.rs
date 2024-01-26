use crate::DB;
use crate::utils::{Context, Error};
use crate::commands::setters::set_timeout_timer::SetTimeoutTimer;
use crate::utils::debug::UnwrapLog;

#[poise::command(prefix_command, slash_command)]
pub async fn get_timeout_timer(
    ctx: Context<'_>,
) -> Result<(), Error> {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let current_line = line!();
    let current_module = module_path!();
    
    let guild_id = ctx.guild_id().unwrap_log("No se pudo obtener el guild_id", current_line, current_module)?;
    let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
    let time_out_timer: Option<SetTimeoutTimer> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id))
        .await?
        .take(0)?;
    
    let Some(time_out_timer) = time_out_timer else {
        poise::say_reply(ctx, "No se ha establecido un tiempo de timeout").await?;
        return Ok(())
    };

    let time = time_out_timer.time;
    poise::say_reply(ctx, format!("The time out timer is set to {time} seconds")).await?;

    Ok(())
}