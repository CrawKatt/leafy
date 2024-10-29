use serenity::all::{GetMessages, Member, UserId};
use crate::utils::Context;
use crate::utils::debug::{IntoUnwrapResult, UnwrapResult};

pub mod generate_furry;
pub mod screenshot_this;
pub mod generate_pride;
pub mod generate_dumb;
pub mod cat;

pub async fn get_target_info(ctx: &Context<'_>, target: Option<Member>) -> UnwrapResult<(UserId, String)> {
    let guild_id = ctx.guild_id().into_result()?; // SAFETY: Si el mensaje no es de un servidor, no se ejecutará el comando

    let Some(target_member) = target else {
        let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
        let message = messages.first().into_result()?;
        let target_id = message.referenced_message.as_ref().into_result()?.author.id;
        let target_member = guild_id.member(&ctx.http(), &target_id).await?;
        let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
        return Ok((target_id, target_avatar))
    };

    let target_avatar = target_member.face(); // el método face devuelve el avatar si existe, de lo contrario, el avatar predeterminado
    let target_id = target_member.user.id;
    Ok((target_id, target_avatar))
}