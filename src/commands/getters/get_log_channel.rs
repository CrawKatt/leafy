use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::commands::setters::GuildData;
use crate::utils::debug::UnwrapLog;

/// Obtiene el canal de logs establecido en el servidor
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn get_log_channel(
    ctx: Context<'_>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let current_line = line!();
    let current_module = module_path!();

    let guild_id = ctx.guild_id().unwrap_log("No se pudo obtener el guild_id", current_module, current_line)?; // obtener el guild_id
    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(database_info) = database_info else {
        ctx.say("No hay un canal de logs establecido").await?;
        return Ok(())
    };

    let log_channel_id = database_info.log_channel_id;
    ctx.say(format!("Log channel is <#{log_channel_id}>")).await?;

    Ok(())
}