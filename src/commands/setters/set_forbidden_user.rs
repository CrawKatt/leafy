use serenity::all::User;
use crate::DB;
use crate::utils::debug::UnwrapLog;
use crate::utils::{CommandResult, Context};
use crate::commands::setters::ForbiddenUserData;
use crate::utils::autocomplete::args_set_forbidden_user;

/// Establece el usuario prohibido de mencionar
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_forbidden_user(
    ctx: Context<'_>,
    #[autocomplete = "args_set_forbidden_user"]
    #[description = "The user to set as the forbidden user"] user: User,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild id", module_path!(), line!())?;
    let data = ForbiddenUserData::new(user.id, guild_id);
    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        data.save_to_db().await?;
        ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;
        return Ok(())
    };

    data.update_in_db().await?;

    ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;

    Ok(())
}