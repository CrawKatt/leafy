use crate::commands::set_forbidden_role::ForbiddenRoleData;
use crate::DB;
use crate::utils::{CommandResult, Context};

#[poise::command(prefix_command, slash_command)]
pub async fn get_forbidden_role(
    ctx: Context<'_>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM forbidden_roles WHERE guild_id = $guild_id";
    let database_info: Option<ForbiddenRoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let forbidden_role_id = database_info.unwrap_or_default().role_id;
    ctx.say(format!("Forbidden role is <@&{}>", forbidden_role_id)).await?;

    Ok(())
}