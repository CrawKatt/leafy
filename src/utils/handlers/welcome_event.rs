use std::collections::HashMap;
use poise::serenity_prelude as serenity;
use serenity::all::ChannelId;
use crate::commands::setters::set_welcome_channel::WelcomeChannelData;
use crate::utils::CommandResult;
use crate::utils::misc::debug::UnwrapLog;

pub async fn welcome_handler(
    ctx: &serenity::Context,
    new_member: &serenity::Member,
) -> CommandResult {
    let guild_id = new_member.guild_id;
    let user = &new_member.user;
    let data = WelcomeChannelData::get_welcome_channel(guild_id).await?;
    let channel_u64 = data.parse::<u64>().unwrap_log("No se pudo convertir el canal de bienvenida a u64", file!(), line!())?;
    let channel_id = ChannelId::new(channel_u64);

    //todo: Crear una función `get_welcome_attachment` que procese y devuelva el archivo de imagen de bienvenida

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("Bienvenido al servidor, {user}"));
    let http = ctx.http.clone();

    // En el attachment se puede pasar un archivo de imagen para la bienvenida
    http.send_message(channel_id, vec![], &message_map).await?;

    Ok(())
}