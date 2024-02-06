use crate::commands::joke::Joke;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

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
    let sql_query = "SELECT * FROM joke WHERE guild_id = $guild_id";
    let database_info: Option<Joke> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let joke = match database_info {
        Some(joke) => joke,
        None => {
            ctx.say("No se ha establecido un usuario prohíbido de mencionar").await?;
            return Ok(())
        }
    };

    let joke_target_id = joke.target.parse::<u64>().unwrap_log("Failed to parse joke target id", module_path!(), line!())?;
    let target = ctx.cache()
        .user(joke_target_id)
        .ok_or("No se ha establecido un usuario prohíbido de mencionar")?
        .name
        .clone();

    ctx.say(format!("Joke is **{}** for **{target}**", joke.is_active)).await?;

    Ok(())
}