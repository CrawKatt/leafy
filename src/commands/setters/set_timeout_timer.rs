use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::misc::autocomplete::args_set_timeout_timer;
use crate::utils::misc::config::{GuildData, TimeOut};
use crate::utils::misc::debug::IntoUnwrapResult;

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
    let guild_id = ctx.guild_id().unwrap();
    let time_in_seconds = match set_time.as_str() {
        "5 Minutos" => "300",
        "30 Minutos" => "1800",
        "60 Minutos" => "3600",
        "1 Semana" => "604_800",
        _ => "60",
    };
    
    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::default()
            .guild_id(guild_id)
            .time_out_config(TimeOut::default()
                .time(time_in_seconds));

        data.save_to_db().await?;
        ctx.say(format!("El tiempo de timeout se ha establecido a {set_time}")).await?;

        return Ok(())
    }

    let data = TimeOut::default()
        .time(time_in_seconds);

    data.update_field_in_db("time_out_config.time", time_in_seconds, &guild_id.to_string()).await?;
    
    let time_out_timer = &*GuildData::verify_data(guild_id).await?
        .into_result()?
        .time_out_config
        .time
        .into_result()?;

    let time_message = match time_out_timer {
        "300" => "5 Minutos",
        "1800" => "30 Minutos",
        "3600" => "60 Minutos",
        "604_800" => "1 Semana",
        _ => "1 Minuto",
    };

    ctx.say(format!("El tiempo de timeout se ha actualizado a {time_message}")).await?;

    Ok(())
}