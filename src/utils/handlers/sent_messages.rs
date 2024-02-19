use serenity::all::{Member, Message, RoleId};
use poise::serenity_prelude as serenity;
use crate::DB;
use crate::commands::joke::Joke;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::commands::setters::ForbiddenUserData;
use crate::commands::setters::ForbiddenRoleData;
use crate::utils::debug::UnwrapLog;
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::handlers::misc::joke_call::handle_joke;

const CURRENT_MODULE: &str = file!();

pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {
    if new_message.author.bot {
        return Ok(());
    }

    // variable que busca la mención en el menssage_content si existe
    let message_content = &new_message.content;

    // variable que obtiene el id del servidor
    let guild_id = new_message.guild_id.unwrap_log("Could not get guild id", CURRENT_MODULE, line!())?;

    if !new_message.attachments.is_empty() {
        for attachment in new_message.attachments.clone() {
            if attachment.content_type.unwrap_or_default().starts_with("audio") {
                let audio_url = &attachment.url;
                let data = MessageData::new(
                    new_message.id,
                    audio_url.to_owned(),
                    new_message.author.id,
                    new_message.channel_id,
                    new_message.guild_id,
                    None
                );

                // Guardar el enlace del archivo de audio en la base de datos
                let _created: Vec<MessageData> = DB.create("audio").content(data).await?;
                println!("Audio file saved to database");
            }
        }
    }

    // Obtener el canal de logs de la base de datos
    let data = MessageData::new(
        new_message.id,
        message_content.to_owned(),
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
        None
    );

    // inicio broma
    let sql_query = "SELECT * FROM joke WHERE guild_id = $guild_id";
    let joke: Option<Joke> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    // match necesario para continuar la función en caso de que el canal de broma no esté establecido
    if let Some(joke) = joke {
        match handle_joke(joke, new_message, ctx).await {
            Ok(_) => (),
            Err(why) => {
                println!("Error handling joke: {why:?}");
            },
        }
    }

    // fin broma

    // Si el mensaje no contiene una mención, guardar el mensaje en la base de datos
    // (NECESARIO PARA EVITAR EL PANIC)
    if !message_content.contains("<@") {
        let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
        return Ok(());
    }

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    // Fallo aquí (se obtiene cualquier user_id y se considera un usuario prohibido de mencionar)
    let forbidden_user_id = ForbiddenUserData::get_forbidden_user_id(guild_id).await?;
    //let forbidden_user_id = forbidden_user_data.user_id.parse::<u64>().ok();
    println!("forbidden_user_id: {:?}", forbidden_user_id);

    if let Some(forbidden_user_id) = forbidden_user_id {
        // Si el usuario prohibido de mencionar es mencionado, silenciar al autor del mensaje
        if new_message.mentions_user_id(forbidden_user_id) {
            handle_forbidden_user(ctx, new_message, guild_id, data, forbidden_user_id).await?;
            return Ok(());
        }
    }

    let get_role_id = ForbiddenRoleData::get_role_id(guild_id).await?;
    let forbidden_role_id = get_role_id.unwrap_log("No se ha establecido un rol prohibido de mencionar", CURRENT_MODULE, line!())?;
    let mentioned_user = guild_id.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get mentioned user roles", CURRENT_MODULE, line!())?;

    // Si el usuario mencionado tiene el rol de prohibido de mencionar, silenciar al autor del mensaje
    if mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id) {
        handle_forbidden_role(ctx, new_message, guild_id, data).await?;
        return Ok(());
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;

    Ok(())
}

pub fn check_admin_exception(admin_role_id: Option<String>, member: &Member, ctx: &serenity::Context) -> bool {
    admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles", CURRENT_MODULE, line!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == RoleId::new(admin_role_id.parse::<u64>().unwrap_or_default()))
    })
}