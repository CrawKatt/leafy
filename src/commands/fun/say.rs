use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Fun",
    guild_only,
    ephemeral,
    description_localized("es-ES", "Repite el mensaje que se le pases como argumento"),
    description_localized("en-US", "Repeats the message you pass as an argument"),
    description_localized("ja", "引数として渡されたメッセージを繰り返します")
)]
pub async fn say(ctx: Context<'_>, msg: String) -> CommandResult {
    ctx.say(msg).await?;
    Ok(())
}