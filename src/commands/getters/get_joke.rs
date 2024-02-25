use crate::commands::setters::set_joke::Joke;
use crate::DB;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral
)]
pub async fn get_joke(
    ctx: Context<'_>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let joke_status = Joke::get_joke_status(guild_id).await?;
    let joke_target_id = Joke::get_joke_target_id(guild_id).await?;
    let target = ctx.cache()
        .user(joke_target_id)
        .ok_or("No se ha establecido un usuario prohíbido de mencionar")?
        .name
        .clone();

    ctx.say(format!("Joke is **{joke_status}** for **{target}**")).await?;

    Ok(())
}