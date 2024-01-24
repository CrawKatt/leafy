use serenity::all::{MessageUpdateEvent, Role, RoleId, UserId};
use poise::serenity_prelude as serenity;
use crate::commands::setters::set_forbidden_role::ForbiddenRoleData;
use crate::commands::setters::set_forbidden_user::ForbiddenUserData;
use crate::commands::setters::set_log_channel::GuildData;
use crate::DB;
use crate::utils::debug::UnwrapLog;
use crate::utils::Error;
use crate::utils::embeds::{edit_message_embed, edit_message_embed_if_mention};
use crate::utils::handlers::sent_messages::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::MessageData;

pub async fn edited_message_handler(ctx: &serenity::Context, event: &MessageUpdateEvent) -> Result<(), Error> {
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
    let new_content = event.content.as_deref().unwrap_log("No se pudo obtener el contenido del mensaje", line!(), module_path!())?;

    if old_content == new_content {
        return Ok(());
    }

    let log_channel_database = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let log_channel_id: Option<GuildData> = DB
        .query(log_channel_database)
        .bind(("guild_id", database_message.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let log_channel = log_channel_id.unwrap_or_default().log_channel_id;
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
    let user_mentioned = user.to_user(&ctx.http).await.unwrap_log("No se pudo obtener el usuario", line!(), module_path!())?;
    let forbidden_user_data = ForbiddenUserData::new(user_mentioned.clone(), user, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", line!(), module_path!())?);
    let forbidden_user_id = forbidden_user_data.user_id;
    let forbiden_role_data = ForbiddenRoleData::new(Role::default(), RoleId::default(), database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", line!(), module_path!())?);
    let result = forbiden_role_data.get_role_id().await?;
    let forbidden_role_id = result.unwrap_log("No se pudo obtener el id del rol", line!(), module_path!())?;
    let mentioned_user = database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", line!(), module_path!())?.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get the roles", line!(), module_path!())?;

    if new_content.contains(&format!("<@{forbidden_user_id}>")) {
        let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
        handle_forbidden_user(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", line!(), module_path!())?,database_message, forbidden_user_id).await?;
    } else if mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id) {
        let message = ctx.http.get_message(database_message.channel_id, database_message.message_id).await?;
        handle_forbidden_role(ctx, &message, database_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", line!(), module_path!())?,database_message).await?;
    } else {
        edit_message_embed_if_mention(ctx, log_channel, &database_message.channel_id, database_message.author_id, &message_content,user_mentioned).await;
    }

    Ok(())
}