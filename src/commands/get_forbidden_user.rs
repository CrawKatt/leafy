use crate::commands::set_forbidden_user::ForbiddenUserData;
use crate::DB;
use crate::utils::{CommandResult, Context};

#[poise::command(prefix_command, slash_command)]
pub async fn get_forbidden_user(
    ctx: Context<'_>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM forbidden_users WHERE guild_id = $guild_id";
    let database_info: Option<ForbiddenUserData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let forbidden_user_id = database_info.unwrap_or_default().user_id;
    ctx.say(format!("Forbidden user is <@{}>", forbidden_user_id)).await?;

    Ok(())
}