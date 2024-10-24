use crate::utils::{Context, Error};
use crate::utils::config::load_data;

/// Obtiene el tiempo de timeout establecido en el servidor.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_timeout_timer(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let time = load_data().time_out.time.parse::<i64>()?;
    poise::say_reply(ctx, format!("The time out timer is set to {time} seconds")).await?;

    Ok(())
}