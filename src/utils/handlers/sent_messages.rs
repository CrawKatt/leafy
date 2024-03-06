use serenity::all::{GuildId, Member, Message};
use poise::serenity_prelude as serenity;
use regex::Regex;
use crate::commands::setters::set_to_blacklist::BlackListData;
use crate::DB;
use crate::commands::setters::set_joke::Joke;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::commands::setters::{AdminData, ForbiddenUserData, SetTimeoutTimer};
use crate::commands::setters::ForbiddenRoleData;
use crate::utils::misc::debug::UnwrapLog;
use crate::utils::handlers::misc::attachment_case::attachment_handler;
use crate::utils::handlers::misc::everyone_case::handle_everyone;
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::handlers::misc::joke_call::handle_joke;

const CURRENT_MODULE: &str = file!();

pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {
    // Si el autor del mensaje es un Bot, no hace falta hacer ninguna acción
    if new_message.author.bot {
        return Ok(());
    }

    // Obtenemos los datos necesarios para guardar el mensaje en la base de datos y para manejar las menciones
    let message_content = &new_message.content;
    let author_user_id = new_message.author.id;
    let guild_id = new_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", CURRENT_MODULE, line!())?;
    let mut member = guild_id.member(&ctx.http, author_user_id).await?;
    let user_id = new_message.mentions.first().map(|user| user.id);

    // Obtenemos el rol de administrador y el tiempo de silencio del servidor desde la base de datos
    let admin_role_id = AdminData::get_admin_role(guild_id).await?;
    let time_out_timer = SetTimeoutTimer::get_time_out_timer(guild_id).await?;
    let time = time_out_timer.unwrap_or_default(); // SAFETY: Si se establece en 0, es porque no se ha establecido un tiempo de silencio

    // Si hay un error al manejar un archivo adjunto, imprimir el error pero no terminar la función
    if let Err(why) = attachment_handler(new_message).await {
        println!("Error handling attachment: {why:?} {CURRENT_MODULE} : {}", line!());
    }

    // Crear un objeto MessageData con la información del mensaje
    let data = MessageData::new(
        new_message.id,
        message_content.to_owned(),
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
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
            Ok(()) => (),
            Err(why) => {
                println!("Error handling joke: {why:?}");
            },
        }
    }

    // fin broma

    // Extraer el link del mensaje si existe
    if let Some(link) = extract_link(&new_message.content) {
        handle_blacklist_link(ctx, new_message, guild_id, link, &data, &admin_role_id, &mut member, time).await?;
    }

    if user_id.is_some() {
        // Si el mensaje contiene una mención a un usuario prohibido, silenciar al autor del mensaje
        let forbidden_user_id = ForbiddenUserData::get_forbidden_user_id(guild_id).await?;
        if let Some(forbidden_user_id_some) = forbidden_user_id {
            if new_message.mentions_user_id(forbidden_user_id_some) {
                handle_forbidden_user(ctx, new_message, guild_id, &data, forbidden_user_id_some).await?;
                let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;
                return Ok(());
            }
        }

        // Si el mensaje contiene una mención a un rol prohibido, silenciar al autor del mensaje
        let database_data = ForbiddenRoleData::get_role_id(guild_id).await?;
        let forbidden_role_id = database_data.unwrap_log("No se ha establecido un rol prohibido de mencionar", CURRENT_MODULE, line!())?;
        let mentioned_user = guild_id.member(&ctx.http, user_id.unwrap()).await?;
        let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get mentioned user roles", CURRENT_MODULE, line!())?;

        if mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id) {
            handle_forbidden_role(ctx, new_message, guild_id, &data).await?;
            let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;
            return Ok(());
        }
    }

    // @everyone no tiene id, por lo que no es necesario el <@id> UBICAR ANTES DE LA CONDICIÓN DE MENCIONES
    if new_message.mention_everyone {
        let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;
        handle_everyone(admin_role_id, &mut member, ctx, time, new_message).await?;

        return Ok(())
    }

    let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;

    Ok(())
}

pub fn extract_link(text: &str) -> Option<String> {
    Regex::new(r"(https?://\S+)").map_or(None, |url_re| url_re.find(text).map(|m| m.as_str().to_string()))
}

async fn handle_blacklist_link(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    link: String,
    data: &MessageData,
    admin_role_id: &Option<String>,
    member: &mut Member,
    time: i64
) -> CommandResult {
    let blacklist_link = BlackListData::get_blacklist_link(guild_id, &link).await?;
    if let Some(blacklist_link) = blacklist_link {
        if new_message.content.contains(&blacklist_link) {
            let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
            handle_everyone(admin_role_id.to_owned(), member, ctx, time, new_message).await?;
            return Ok(());
        }
    }
    Ok(())
}