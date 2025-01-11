use serenity::all::{Member, Message, Timestamp};
use poise::serenity_prelude as serenity;
use crate::utils::CommandResult;
use crate::handlers::misc::exceptions::check_admin_exception;

/// Silencia al autor del mensaje y elimina el mensaje
pub async fn handle_everyone(
    admin_role_id: Option<&Vec<String>>,
    member: &mut Member,
    ctx: &serenity::Context,
    time_out_timer: i64,
    message: &Message,
) -> CommandResult {

    if check_admin_exception(admin_role_id, member, ctx) { return Ok(()) }
    let time = Timestamp::now().unix_timestamp() + time_out_timer;
    let time = Timestamp::from_unix_timestamp(time)?;

    member.disable_communication_until_datetime(&ctx.http, time).await?;
    message.delete(&ctx.http).await?;

    Ok(())
}