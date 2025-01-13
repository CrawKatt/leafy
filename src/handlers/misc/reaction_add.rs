use poise::serenity_prelude as serenity;
use regex::Regex;
use serenity::all::{ChannelId, Message, Reaction, ReactionType, RoleId};

use crate::utils::CommandResult;
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapResult};
use crate::utils::config::AutoRole;

pub async fn handler(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    autorole(ctx, add_reaction).await?;
    vote_react(ctx, add_reaction).await?;
    Ok(())
}

pub async fn autorole(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    if add_reaction.user(&ctx.http).await?.bot { return Ok(()) }
    let guild_id = add_reaction.guild_id.ok_or("error")?;
    let emoji = add_reaction.emoji.to_string();
    let emoji_id = Regex::new(r"^<a?:\w+:(\d+)>")?
        .captures(&emoji)
        .into_result()?
        .get(1)
        .into_result()?
        .as_str()
        .to_string();

    let data = AutoRole::get_assignments(guild_id).await?;
    let Some(data) = data else {
        println!("[ERROR]: No se encontraron asignaciones de autoroles para este servidor.");
        return Ok(());
    };

    if let Some(assignment) = data
        .assignments
        .iter()
        .find(|a| a.emoji_id == emoji_id)
    {
        let role_id: RoleId = assignment.role.parse()?;
        let user = add_reaction.user_id.ok_or("UserID not found")?;
        let member = guild_id.member(ctx, user).await?;
        member.add_role(ctx, role_id).await?;
    }

    Ok(())
}

/// # Esta función maneja las reacciones con un sistema de votación
///
/// - Si la reacción es igual a ❌ y tiene 5 o más reacciones, se elimina el mensaje
/// - La función solo se activa si el mensaje está en el canal OOC
pub async fn vote_react(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    let guild_id = add_reaction.guild_id.ok_or("error")?;
    let message = add_reaction.message(&ctx.http).await?;
    let target_emoji_negative = ReactionType::from('🔻');
    let target_emoji_positive = ReactionType::from('🔺');

    let channel_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channels
        .ooc
        .into_result()?
        .parse::<ChannelId>()?;

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