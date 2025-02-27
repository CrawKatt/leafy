use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{Context, Error};
use crate::utils::debug::IntoUnwrapResult;

/// Obtiene el tiempo de timeout establecido en el servidor.
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
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().into_result()?;
    let time_out_timer: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;
    
    let Some(time_out_timer) = time_out_timer else {
        poise::say_reply(ctx, "No se ha establecido un tiempo de timeout").await?;
        return Ok(())
    };

    let time = time_out_timer
        .time_out
        .time
        .ok_or("No se encontró un tiempo de timeout o no ha sido establecido")?;
    
    poise::say_reply(ctx, format!("The time out timer is set to {time} seconds")).await?;

    Ok(())
}