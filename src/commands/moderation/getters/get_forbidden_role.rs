use serenity::all::RoleId;
use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{CommandResult, Context};

/// Obtiene el rol que está prohibido de mencionar si está establecido.
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
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(forbidden_role_id) = database_info else {
        ctx.say("No se ha establecido un rol prohíbido de mencionar").await?;
        return Ok(())
    };
    
    let forbidden_role_id = forbidden_role_id
        .forbidden
        .role
        .ok_or("No se ha establecido un rol prohíbido de mencionar")?
        .parse::<RoleId>()?;
    
    let guild = ctx.cache().guild(guild_id).ok_or("Guild not found")?.clone();
    let forbidden_role = guild.roles.get(&forbidden_role_id).ok_or("Role not found")?;
    let forbidden_role_name = &*forbidden_role.name;

    ctx.say(format!("Forbidden role is **{forbidden_role_name}**")).await?;

    Ok(())
}