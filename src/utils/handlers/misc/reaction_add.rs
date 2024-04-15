use poise::serenity_prelude as serenity;
use serenity::all::{Reaction, ReactionType};

use crate::utils::CommandResult;
use crate::commands::setters::set_ooc_channel::OocChannel;


/// # Esta función maneja las reacciones con un sistema de votación
///
/// - Si la reacción es igual a ❌ y tiene 2 o más reacciones, se elimina el mensaje
/// - La función solo se activa si el mensaje está en el canal OOC
pub async fn vote_react(ctx: &serenity::Context, add_reaction: &Reaction) -> CommandResult {
    let guild_id = add_reaction.guild_id.ok_or("error")?;
    let message = add_reaction.message(&ctx.http).await?;
    let tarjet_emoji = ReactionType::try_from('❌')?;

    let sql_query = "SELECT * FROM ooc_channel WHERE guild_id = $guild_id";
    let existing_data: Option<OocChannel> = crate::DB
        .query(sql_query)
        .bind(("guild_id", &guild_id.to_string()))
        .await?
        .take(0)?;

    let ooc_channel = existing_data.ok_or("No se pudo obtener el canal OOC o no ha sido establecido")?;
    let channel_u64 = ooc_channel.channel_id.parse::<u64>()?;

    // Filtra si el emoji es el correcto y si el mensaje está en el canal OOC
    // Nota: (Se hace uso de cláusulas de guardia para evitar el anidamiento de if)
    if add_reaction.emoji != tarjet_emoji || message.channel_id != channel_u64 {
        return Ok(())
    }

    let reaction_count = message
        .reaction_users(&ctx.http, add_reaction.emoji.clone(), None, None)
        .await?
        .len();

    if reaction_count >= 5 {
        message.delete(&ctx.http).await?;
    }

    Ok(())
}