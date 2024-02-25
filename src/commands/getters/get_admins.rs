use serenity::all::RoleId;
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
    let role_id_1 = AdminData::get_admin_role(guild_id).await?;
    let role_id_2 = AdminData::get_admin_role_2(guild_id).await?; // Obtén el segundo rol de administrador

    let mut role_names = String::new();

    if let Some(role_id) = role_id_1 {
        let parse = role_id.parse::<u64>().unwrap();
        let role_id = RoleId::new(parse);
        let role_name = ctx.cache().role(guild_id, role_id).ok_or("No se han establecido roles de moderador")?.name.clone();
        role_names.push_str(&role_name);
    }

    if let Some(role_id) = role_id_2 {
        let parse = role_id.parse::<u64>().unwrap();
        let role_id = RoleId::new(parse);
        let role_name = ctx.cache().role(guild_id, role_id).ok_or("No se han establecido roles de moderador")?.name.clone();
        if !role_names.is_empty() {
            role_names.push_str(", ");
        }
        role_names.push_str(&role_name);
    }

    if role_names.is_empty() {
        poise::say_reply(ctx, "No hay roles de administrador establecidos").await?;
        return Ok(())
    };

    poise::say_reply(ctx, format!("Los roles de administrador actuales son: **{role_names}**")).await?;

    Ok(())
}