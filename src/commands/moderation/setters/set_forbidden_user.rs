use serenity::all::User;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_forbidden_user;
use crate::utils::config::{GuildData, Forbidden};

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
    #[description = "The user to set as the forbidden user"]
    #[autocomplete = "args_set_forbidden_user"]
    forbidden_user: User,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let user_id = forbidden_user.id.to_string();

    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::default()
            .guild_id(guild_id)
            .forbidden(Forbidden::default()
                .user(user_id)
            );

        data.save_to_db().await?;
        ctx.say(format!("Se ha prohibido mencionar a: **{}**", forbidden_user.name)).await?;
        return Ok(())
    }

    let data = Forbidden::default().user(&user_id);
    data.update_field_in_db("forbidden.user", &user_id, &guild_id.to_string()).await?;
    ctx.say(format!("Se ha prohibido mencionar a: **{}**", forbidden_user.name)).await?;

    Ok(())
}