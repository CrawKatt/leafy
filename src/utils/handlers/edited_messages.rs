use serenity::all::{ChannelId, MessageUpdateEvent};
use poise::serenity_prelude as serenity;
use crate::utils::misc::config::GuildData;
use crate::utils::misc::debug::{IntoUnwrapResult, UnwrapLog};
use crate::utils::CommandResult;
use crate::utils::misc::embeds::{edit_message_embed};
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_user, handle_forbidden_role};
use crate::utils::MessageData;
use crate::match_handle;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> CommandResult {
    let current_module = file!();

    if event.author.as_ref().map_or(false, |author| author.bot) {
        return Ok(());
    }

    let message_id = event.id;
    let guild_id = event.guild_id.unwrap_or_default(); // SAFETY: El GuildId siempre está disponible
    let old_message = MessageData::get_message_data(&message_id).await?;
    let Some(database_message) = old_message else { return Ok(()) };

    let old_content = &database_message.message_content;
    let new_content = event.content.as_deref().into_result()?;
    if old_content == new_content { return Ok(()) }

    //let result_database = database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?;
    //let log_channel_id = GuildData::get_log_channel(result_database).await?;

    //let log_channel = log_channel_id.unwrap_log("No se pudo obtener el canal de Logs", current_module, line!())?;
    let log_channel = GuildData::verify_data(guild_id).await?
        .into_result()?
        .channel_config
        .log_channel_id
        .into_result()?
        .parse::<ChannelId>()?;
    
    let message_content = format!("\n**Antes:** \n> {old_content}\n**Después:** \n> {new_content}");

    // Bug: Resolver, un unwrap_or_default() está devolviendo 1
    let user_id = event.mentions
        .clone()
        .unwrap_or_default()
        .first()
        .map(|user| user.id);

    if let Some(user_id) = user_id {
        //let forbidden_user_id = ForbiddenUserData::get_forbidden_user_id(guild_id).await?;
        //let forbidden_user_id = forbidden_user_id.unwrap_log("No se pudo obtener el id del usuario", current_module, line!())?;
        let forbidden_user_id = GuildData::verify_data(guild_id).await?
            .into_result()?
            .forbidden_config
            .user_id
            .into_result()?
            .parse::<u64>()?;

        //let forbidden_role_id = ForbiddenRoleData::get_role_id(guild_id).await?;
        //let forbidden_role_id = forbidden_role_id.unwrap_log("No se pudo obtener el id del rol prohíbido o no está configurado", current_module, line!())?;
        let forbidden_role_id = GuildData::verify_data(guild_id).await?
            .into_result()?
            .forbidden_config
            .role_id
            .into_result()?
            .parse::<u64>()?;
        
        let mentioned_user = guild_id.member(&ctx.http, user_id).await?; // SAFETY: El GuildId siempre está disponible
        let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get the roles", current_module, line!())?;

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
    }

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