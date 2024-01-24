use crate::commands::setters::set_timeout_role::RoleData;
use crate::DB;
use crate::utils::{CommandResult, Context};

/// Obtiene el rol de timeout establecido en el servidor
#[poise::command(prefix_command, slash_command)]
pub async fn get_timeout_role(
    ctx: Context<'_>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
    let database_info: Option<RoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let timeout_role_id = database_info.unwrap_or_default().role_id;
    ctx.say(format!("Timeout role is <@&{}>", timeout_role_id)).await?;

    Ok(())
}