use serenity::all::{MessageUpdateEvent, UserId};
use poise::serenity_prelude as serenity;
use crate::commands::setters::ForbiddenRoleData;
use crate::commands::setters::ForbiddenUserData;
use crate::commands::setters::GuildData;
use crate::DB;
use crate::utils::debug::UnwrapLog;
use crate::utils::Error;
use crate::utils::embeds::{edit_message_embed, edit_message_embed_if_mention};
use crate::utils::handlers::sent_messages::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::MessageData;
use crate::matches;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> Result<(), Error> {
    let current_module = file!();

    if event.author.as_ref().map_or(false, |author| author.bot) {
        return Ok(());
    }

    let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
    let old_message: Option<MessageData> = DB
        .query(sql_query)
        .bind(("message_id", event.id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(database_message) = old_message else {
        return Ok(())
    };

    let old_content = &database_message.message_content;
    let new_content = event.content.as_deref().unwrap_log("No se pudo obtener el contenido del mensaje", current_module, line!())?;

    if old_content == new_content {
        return Ok(());
    }

    let log_channel_database = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(log_channel_database)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_log("No se pudo obtener el canal de Logs", current_module, line!())?.log_channel_id;
    let message_content = format!("\n**Antes:** {old_content}\n**Después:** {new_content}");

    if !message_content.contains("<@") {
        edit_message_embed(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content).await;
        return Ok(());
    }

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    let user = UserId::new(user_id);
    let user_mentioned = user.to_user(&ctx.http).await.unwrap_log("No se pudo obtener el usuario", current_module, line!())?;
    let forbidden_user_data = ForbiddenUserData::new(u64::from(user).into(), database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?);
    let forbidden_user_id = forbidden_user_data.user_id;
    let forbidden_role_id = ForbiddenRoleData::get_role_id(database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?).await?;
    let forbidden_role_id = forbidden_role_id.unwrap_log("No se pudo obtener el id del rol", current_module, line!())?;
    let mentioned_user = database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get the roles", current_module, line!())?;

    let contains_forbidden_user = new_content.contains(&format!("<@{forbidden_user_id}>"));
    let contains_forbidden_role = mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id);

    matches!(
        contains_forbidden_user, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_user(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?,database_message, forbidden_user_id.parse::<u64>()?).await?;
        },
        contains_forbidden_role, {
            let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
            handle_forbidden_role(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", current_module, line!())?,database_message).await?;
        },
        default, {
            edit_message_embed_if_mention(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content,user_mentioned).await;
        }
    );

    Ok(())
}

#[macro_export]
macro_rules! matches {
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