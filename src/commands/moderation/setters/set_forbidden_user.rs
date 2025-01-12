use serenity::all::User;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{Forbidden, GuildData, DatabaseOperations};

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
    forbidden_user: User,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let user_id = forbidden_user.id.to_string();

    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::builder()
            .forbidden(Forbidden::builder()
                .user(&user_id)
                .build()
            )
            .build();

        data.save_to_db(guild_id).await?;
        ctx.say(format!("Se ha prohibido mencionar a: **{}**", forbidden_user.name)).await?;
        return Ok(())
    }

    let data = Forbidden::builder()
        .user(&user_id)
        .build();

    data.update_field_in_db("forbidden/user", &user_id, &guild_id.to_string()).await?;
    ctx.say(format!("Se ha prohibido mencionar a: **{}**", forbidden_user.name)).await?;

    Ok(())
}