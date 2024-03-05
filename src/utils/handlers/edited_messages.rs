use serenity::all::MessageUpdateEvent;
use poise::serenity_prelude as serenity;
use crate::commands::setters::ForbiddenRoleData;
use crate::commands::setters::ForbiddenUserData;
use crate::commands::setters::GuildData;
use crate::utils::misc::debug::UnwrapLog;
use crate::utils::Error;
use crate::utils::misc::embeds::{edit_message_embed};
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_user, handle_forbidden_role};
use crate::utils::MessageData;
use crate::match_handle;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> Result<(), Error> {
    let current_module = file!();

    if event.author.as_ref().map_or(false, |author| author.bot) {
        return Ok(());
    }

    let message_id = event.id;
    let old_message = MessageData::get_message_data(&message_id).await?;

    let Some(database_message) = old_message else {
        return Ok(())
    };

    let old_content = &database_message.message_content;
    let new_content = event.content.as_deref().unwrap_log("No se pudo obtener el contenido del mensaje", current_module, line!())?;

    if old_content == new_content {
        return Ok(());
    }

    let result_database = database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?;
    let log_channel_id = GuildData::get_log_channel(result_database).await?;

    let log_channel = log_channel_id.unwrap_log("No se pudo obtener el canal de Logs", current_module, line!())?.log_channel_id;
    let message_content = format!("\n**Antes:** {old_content}\n**Después:** {new_content}");

    if !message_content.contains("<@") {
        edit_message_embed(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await?;
        return Ok(());
    }

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    let forbidden_user_id = ForbiddenUserData::get_forbidden_user_id(database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?).await?;
    let forbidden_user_id = forbidden_user_id.unwrap_log("No se pudo obtener el id del usuario", current_module, line!())?;

    let guild_id = event.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?;
    let forbidden_role_id = ForbiddenRoleData::get_role_id(guild_id).await?;
    let forbidden_role_id = forbidden_role_id.unwrap_log("No se pudo obtener el id del rol prohíbido o no está configurado", current_module, line!())?;
    let mentioned_user = database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get the roles", current_module, line!())?;

    let contains_forbidden_user = new_content.contains(&format!("<@{forbidden_user_id}>"));
    let contains_forbidden_role = mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id);

    match_handle!(
        contains_forbidden_user, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_user(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?,&database_message, forbidden_user_id).await?;
        },
        contains_forbidden_role, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_role(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?,&database_message).await?;
        },
        default, {
            edit_message_embed(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await?;
        }
    );

    Ok(())
}

#[macro_export]
macro_rules! match_handle {
    ($cond1:expr, $block1:block, $cond2:expr, $block2:block, default, $block3:block) => {
        if $cond1 {
            $block1
        } else if $cond2 {
            $block2
        } else {
            $block3
        }
    };
}