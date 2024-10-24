use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, Message, Reaction, ReactionType};

use crate::utils::config::load_data;
use crate::utils::debug::UnwrapResult;
use crate::utils::CommandResult;

pub async fn handler(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    vote_react(ctx, add_reaction).await?;
    Ok(())
}

/// # Esta función maneja las reacciones con un sistema de votación
///
/// - Si la reacción es igual a ❌ y tiene 5 o más reacciones, se elimina el mensaje
/// - La función solo se activa si el mensaje está en el canal OOC
pub async fn vote_react(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    let message = add_reaction.message(&ctx.http).await?;
    let target_emoji_negative = ReactionType::from('🔻');
    let target_emoji_positive = ReactionType::from('🔺');
    let channel_id = load_data().channels.ooc.parse::<ChannelId>()?;

    if message.channel_id != channel_id { return Ok(()) }
    let (reaction_count_positive, reaction_count_negative) = get_reaction_counts(ctx, &message, target_emoji_positive, target_emoji_negative).await?;
    let message_approved = reaction_count_positive >= 5;
    if reaction_count_negative >= 5 && !message_approved {
        message.delete(&ctx.http).await?;
    }

    Ok(())
}

/// # Esta función obtiene el conteo de reacciones
///
/// - Obtiene el conteo de reacciones positivas y negativas
async fn get_reaction_counts(
    ctx: &serenity::Context,
    message: &Message,
    target_emoji_positive: ReactionType,
    target_emoji_negative: ReactionType
) -> UnwrapResult<(usize, usize)> {
    let reaction_count_negative = message
        .reaction_users(&ctx.http, target_emoji_negative, None, None)
        .await?
        .len();

    let reaction_count_positive = message
        .reaction_users(&ctx.http, target_emoji_positive, None, None)
        .await?
        .len();

    Ok((reaction_count_positive, reaction_count_negative))
}