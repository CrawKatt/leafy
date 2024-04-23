use poise::serenity_prelude as serenity;
use serenity::all::{Message, Reaction, ReactionType};

use crate::utils::CommandResult;
use crate::commands::setters::set_ooc_channel::OocChannel;
use crate::utils::misc::debug::UnwrapResult;

/// # Esta función maneja las reacciones con un sistema de votación
///
/// - Si la reacción es igual a ❌ y tiene 2 o más reacciones, se elimina el mensaje
/// - La función solo se activa si el mensaje está en el canal OOC
pub async fn vote_react(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    let guild_id = add_reaction.guild_id.ok_or("error")?;
    let message = add_reaction.message(&ctx.http).await?;
    let target_emoji_negative = ReactionType::from('🔻');
    let target_emoji_positive = ReactionType::from('🔺');

    let sql_query = "SELECT * FROM ooc_channel WHERE guild_id = $guild_id";
    let existing_data: Option<OocChannel> = crate::DB
        .query(sql_query)
        .bind(("guild_id", &guild_id.to_string()))
        .await?
        .take(0)?;

    let ooc_channel = existing_data.ok_or("No se pudo obtener el canal OOC o no ha sido establecido")?;
    let channel_u64 = ooc_channel.channel_id.parse::<u64>()?;

    if message.channel_id != channel_u64 {
        return Ok(())
    }

    let (reaction_count_positive, reaction_count_negative) = get_reaction_counts(ctx, &message, target_emoji_positive, target_emoji_negative).await?;
    
    let mut message_approved = false;
    if reaction_count_positive >= 5 {
        message_approved = true;
    }

    if reaction_count_negative >= 5 && !message_approved {
        message.delete(&ctx.http).await?;
    }

    Ok(())
}

/// # Esta función obtiene el conteo de reacciones
///
/// - Obtiene el conteo de reacciones positivas y negativas
async fn get_reaction_counts(ctx: &serenity::Context, message: &Message, target_emoji_positive: ReactionType, target_emoji_negative: ReactionType) -> UnwrapResult<(usize, usize)> {
    let reaction_count_negative = message
        .reaction_users(&ctx.http, target_emoji_negative.clone(), None, None)
        .await?
        .len();

    let reaction_count_positive = message
        .reaction_users(&ctx.http, target_emoji_positive.clone(), None, None)
        .await?
        .len();

    Ok((reaction_count_positive, reaction_count_negative))
}