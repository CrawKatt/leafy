use serenity::all::Role;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_admins;
use crate::utils::debug::UnwrapLog;
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
    let current_module = file!();
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id", current_module, line!())?;
    let role_id = role.as_ref().map(|role| role.id);
    let role_2_id = role_2.as_ref().map(|role| role.id);

    let admin_data = AdminData::new(role_id, role_2_id, guild_id);
    admin_data.save_to_db().await?;

    let role_name = role.unwrap_log("Could not get the role_name", module_path!(), line!())?.name;
    ctx.say(format!("Admin role set to: **{role_name}**")).await?;
    
    let role_2 = role_2.unwrap_log("Could not get the role", module_path!(), line!())?;
    let role_2_name = role_2.name;
    ctx.say(format!("Admin role set to: **{role_2_name}**")).await?;

    Ok(())
}