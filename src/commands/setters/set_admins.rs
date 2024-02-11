use serenity::all::Role;
use crate::{DB, unwrap_log};
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_admins;
use crate::commands::setters::AdminData;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn set_admins(
    ctx: Context<'_>,
    #[description = "El rol para establecer como administrador"]
    #[autocomplete = "args_set_admins"]
    role: Option<Role>,
    #[description = "El segundo rol para establecer como administrador (opcional)"]
    role_2: Option<Role>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = unwrap_log!(ctx.guild_id(), "No se pudo obtener el guild_id");
    let role_id = role.as_ref().map(|role| role.id);
    let role_2_id = role_2.as_ref().map(|role| role.id);

    let admin_data = AdminData::new(role_id, role_2_id, guild_id);
    admin_data.save_to_db().await?;

    let role_name = unwrap_log!(role.clone(), "No se pudo obtener el nombre del rol").name;
    ctx.say(format!("Admin role set to: **{role_name}**")).await?;
    
    let role_2 = unwrap_log!(role, "No se pudo obtener el nombre del segundo rol").name;
    ctx.say(format!("Admin role set to: **{role_2}**")).await?;

    Ok(())
}