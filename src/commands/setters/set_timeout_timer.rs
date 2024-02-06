use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_timeout_timer;
use crate::utils::debug::UnwrapLog;
use crate::commands::setters::SetTimeoutTimer;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_timeout_timer(
    ctx: Context<'_>,
    #[autocomplete = "args_set_timeout_timer"]
    #[description = "The time to set as the time out timer"] set_time: String,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    
    let guild_id = ctx.guild_id().unwrap_log("Failed to get guild id", module_path!(), line!())?;
    let time_in_seconds = match set_time.as_str() {
        "5 Minutos" =>300,
        "30 Minutos" => 1800,
        "60 Minutos" => 3600,
        "1 Semana" => 604_800,
        _ => 60,
    };

    let time_out_timer = SetTimeoutTimer::new(time_in_seconds, guild_id);
    let mut existing_data = time_out_timer.verify_data().await?;

    let time_message = match time_out_timer.time {
        300 => "5 Minutos",
        1800 => "30 Minutos",
        3600 => "60 Minutos",
        604_800 => "1 Semana",
        _ => "1 Minuto",
    };

    let Some(existing_data) = &mut existing_data else {
        time_out_timer.save_to_db().await?;
        ctx.say(format!("El tiempo de timeout se ha establecido a {time_message}")).await?;

        return Ok(())
    };

    existing_data.time = time_in_seconds;
    existing_data.update_in_db().await?;

    poise::say_reply(ctx, format!("El tiempo de timeout se ha actualizado a {time_message}")).await?;

    Ok(())
}