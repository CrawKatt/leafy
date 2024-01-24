use crate::utils::{CommandResult, Context};
use crate::commands::setters::set_admins::AdminData;

/// Obtiene el rol de administrador establecido en el servidor
#[poise::command(prefix_command, slash_command)]
pub async fn get_admins(
    ctx: Context<'_>,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_or_default();
    let role_id = AdminData::get_admin_role(guild_id).await?;

    let Some(role_id) = role_id else {
        poise::say_reply(ctx, "No admin role has been set").await?;
        return Ok(())
    };

    let role_names = &ctx.cache().role(guild_id, role_id).ok_or("Role not found")?.name.clone();
    poise::say_reply(ctx, format!("The current admin role is: **{role_names}**")).await?;

    Ok(())
}