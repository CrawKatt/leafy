use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, MessageUpdateEvent, RoleId, UserId};

use crate::{location, match_handle};
use crate::utils::CommandResult;
use crate::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::MessageData;
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::utils::embeds::edit_message_embed;

pub async fn handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> CommandResult {
    if event.author.as_ref().map_or(false, |author| author.bot) { return Ok(()) }
    let message_id = event.id;
    let guild_id = event.guild_id.unwrap_log(location!())?; // SAFETY: El GuildId siempre está disponible
    let old_message = MessageData::get_message_data(&message_id).await?;
    let Some(database_message) = old_message else { return Ok(()) };

    let old_content = &database_message.message_content;
    let new_content = event.content.as_deref().unwrap_log(location!())?;
    if old_content == new_content { return Ok(()) }
    let log_channel = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .channels
        .logs
        .into_result()?
        .parse::<ChannelId>()?;
    
    let message_content = format!("\n**Antes:** \n> {old_content}\n**Después:** \n> {new_content}");

    let mention = event.mentions.clone().into_result()?;
    let first = mention.first();

    let Some(user) = first else {
        edit_message_embed(ctx, guild_id, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await?;
        return Ok(())
    };
    
    let user_id = user.id;
    let forbidden_user_id = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .forbidden
        .user
        .into_result()?
        .parse::<UserId>()?;

    let forbidden_role_id = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .forbidden
        .role
        .into_result()?
        .parse::<RoleId>()?;

    let mentioned_user = guild_id.member(&ctx.http, user_id).await?; // SAFETY: El GuildId siempre está disponible
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log(location!())?;

    let contains_forbidden_user = new_content.contains(&format!("<@{forbidden_user_id}>"));
    let contains_forbidden_role = mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id);

    match_handle!(
        contains_forbidden_user, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_user(ctx, &message, guild_id, &database_message, forbidden_user_id).await?;
        },
        contains_forbidden_role, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_role(ctx, &message, guild_id).await?;
        }
    );

    edit_message_embed(ctx, guild_id,log_channel, &database_message.channel_id, database_message.author_id, &message_content).await?;

    Ok(())
}

#[macro_export]
macro_rules! match_handle {
    ($cond1:expr, $block1:block, $cond2:expr, $block2:block) => {
        if $cond1 {
            $block1
        } else if $cond2 {
            $block2
        }
    };
}