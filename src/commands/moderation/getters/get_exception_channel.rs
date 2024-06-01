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
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().into_result()?;
    let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

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