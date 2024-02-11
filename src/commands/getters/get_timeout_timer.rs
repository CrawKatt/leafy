use crate::{DB, unwrap_log};
use crate::utils::{Context, Error};
use crate::commands::setters::SetTimeoutTimer;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_timeout_timer(
    ctx: Context<'_>,
) -> Result<(), Error> {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = unwrap_log!(ctx.guild_id(), "No se pudo obtener el guild_id");
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