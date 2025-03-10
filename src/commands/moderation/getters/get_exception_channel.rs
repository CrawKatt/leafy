use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

/// Obtiene el canal de logs establecido en el servidor
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn get_exception_channel(
    ctx: Context<'_>,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().into_result()?;
    let database_info: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;

    let Some(database_info) = database_info else {
        ctx.say("No hay un canal de excepciones establecido").await?;
        return Ok(())
    };

    let exception_channel = database_info
        .channels
        .exceptions
        .ok_or("No se encontró un canal de excepciones o no ha sido establecido")?;

    ctx.say(format!("Exceptions channel is <#{exception_channel}>")).await?;

    Ok(())
}