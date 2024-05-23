use std::panic::Location;
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::all::{GuildId, Message, RoleId, UserId};

use crate::{DB, location};
use crate::utils::CommandResult;
use crate::utils::handlers::misc::attachment_case::attachment_handler;
use crate::utils::handlers::misc::everyone_case::handle_everyone;
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::handlers::misc::link_spam_handler::{extract_link, spam_checker};
use crate::utils::MessageData;
use crate::utils::misc::config::GuildData;
use crate::utils::misc::debug::{IntoUnwrapResult, UnwrapLog};

/// # Esta función maneja los mensajes enviados en un servidor
///
/// ## Funciones relacionadas:
/// - manejo de archivos adjuntos
/// - manejo de menciones a roles y usuarios prohibidos
/// - manejo de menciones a @everyone y @here
/// - manejo de spam de links
/// - guardar el mensaje en la base de datos
pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {

    // Crear un objeto Arc<String> con el contenido del mensaje para utilizar cheap cloning (clonación barata)
    // La clonación barata consiste en utilizar Arc<T> o Rc<T> para clonar un objeto sin copiar su contenido
    // Rc<T> es para usar en hilos de ejecución y Arc<T> es para usar en hilos de ejecución concurrentes (async)
    let message_content = Arc::new(String::from(&new_message.content));
    if new_message.author.bot { return Ok(()) }
    let guild_id = new_message.guild_id.into_result()?;
    let mut member = guild_id.member(&ctx.http, new_message.author.id).await?;
    let user_id = new_message.mentions.first().map(|user| user.id);
    let admin_role_id = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .admins
        .role_id;
    
    let time = GuildData::verify_data(guild_id).await?
        .into_result()?
        .time_out_config
        .time
        .into_result()?
        .parse::<i64>()?;

    // Si hay un error al manejar un archivo adjunto, imprimir el error pero no terminar la función
    if let Err(why) = attachment_handler(new_message).await {
        println!("Error handling attachment: {why:?} {}", Location::caller());
    }

    let data = MessageData::new(
        new_message.id,
        &message_content,
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
    );

    // Extraer el link del mensaje si existe
    if extract_link(&message_content).is_some() {
        let channel_id = new_message.channel_id;
        spam_checker(&message_content, channel_id, &admin_role_id, ctx, time, new_message, guild_id).await?;
    }

    if user_id.is_some() {
        handle_user_id(ctx, new_message, guild_id, &data, user_id).await?;
    }

    // @everyone no tiene id, por lo que no es necesario el <@id>
    // Si bien hay un método para comprobar si se menciona @everyone o @here, este método devuelve
    // `false` en servidores donde @everyone y @here están deshabilitados
    if message_content.contains("@everyone") || message_content.contains("@here") {
        let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;
        handle_everyone(admin_role_id, &mut member, ctx, time, new_message).await?;

        return Ok(())
    }

    let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;

    Ok(())
}

/// # Esta función maneja las menciones a usuarios y roles prohibidos
///
/// ## Funciones relacionadas:
/// - manejo de menciones a usuarios prohibidos
/// - manejo de menciones a roles prohibidos
/// - silenciar al autor del mensaje
/// - guardar el mensaje en la base de datos
async fn handle_user_id(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: &MessageData,
    user_id: Option<UserId>
) -> CommandResult {
    
    let forbidden_user_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .forbidden_config
        .user_id
        .unwrap_log(location!())?
        .parse::<UserId>()?;

    if new_message.mentions_user_id(forbidden_user_id) {
        handle_forbidden_user(ctx, new_message, guild_id, data, forbidden_user_id).await?;
        DB.query("DEFINE INDEX message_id ON TABLE messages COLUMNS message_id UNIQUE").await?;
        let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
        return Ok(())
    }

    let forbidden_role_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .forbidden_config
        .role_id
        .into_result()?
        .parse::<RoleId>()?;
    
    let has_role = user_id
        .into_result()?
        .to_user(&ctx.http).await?
        .has_role(&ctx.http, guild_id, forbidden_role_id).await?;

    if has_role { handle_forbidden_role(ctx, new_message, guild_id).await? };

    Ok(())
}