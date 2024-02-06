use crate::utils::{CommandResult, Context};
use crate::commands::setters::AdminData;
use crate::utils::debug::UnwrapLog;

/// Obtiene el rol de administrador establecido en el servidor
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_admins(
    ctx: Context<'_>,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_log("Error getting guild id", module_path!(), line!())?;
    let role_id = AdminData::get_admin_role(guild_id).await?;

    let Some(role_id) = role_id else {
        poise::say_reply(ctx, "No hay un rol de administrador establecido").await?;
        return Ok(())
    };

    let role_names = ctx.cache().role(guild_id, role_id).ok_or("No se han establecido roles de moderador")?.name.clone();
    poise::say_reply(ctx, format!("El rol de administrador actual es: **{role_names}**")).await?;

    Ok(())
}