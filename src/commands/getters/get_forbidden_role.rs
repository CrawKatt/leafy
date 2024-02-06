use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::commands::setters::ForbiddenRoleData;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
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

    let Some(forbidden_role_id) = database_info else {
        ctx.say("No se ha establecido un rol prohíbido de mencionar").await?;
        return Ok(())
    };

    let forbidden_role_id = forbidden_role_id.role_id.parse::<u64>()?;
    let forbidden_role = ctx.cache().role(guild_id, forbidden_role_id).ok_or("Role not found")?.name.clone();

    ctx.say(format!("Forbidden role is **{forbidden_role}**")).await?;

    Ok(())
}