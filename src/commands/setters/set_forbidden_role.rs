use serenity::all::Role;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_forbidden_role;
use crate::utils::debug::UnwrapLog;
use crate::commands::setters::ForbiddenRoleData;

/// Establece el rol de usuario prohibido de mencionar
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_forbidden_role(
    ctx: Context<'_>,
    #[autocomplete = "args_set_forbidden_role"]
    #[description = "The role to set as the forbidden role"] role: Role,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id", module_path!(), line!())?;
    let data = ForbiddenRoleData::new(role.id, guild_id);
    let existing_data = data.verify_data().await?;
    
    let Some(_) = existing_data else {
        data.save_to_db().await?;
        ctx.say(format!("Set forbidden role to: **{}**", role.name)).await?;
        return Ok(())
    };
    
    data.update_in_db().await?;

    let message = format!("Set forbidden role to: **{}**", role.name);
    ctx.say(message).await?;

    Ok(())
}