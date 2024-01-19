use crate::commands::setters::set_log_channel::GuildData;
use crate::DB;
use crate::utils::{CommandResult, Context};

/// Obtiene el canal de logs establecido en el servidor
#[poise::command(prefix_command, slash_command)]
pub async fn get_log_channel(
    ctx: Context<'_>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel_id = database_info.unwrap_or_default().log_channel_id;
    ctx.say(format!("Log channel is <#{log_channel_id}>")).await?;

    Ok(())
}