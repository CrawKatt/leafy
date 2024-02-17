use std::sync::Arc;
use std::collections::HashMap;
use std::panic::Location;
use chrono::{Duration, Utc};
use serenity::all::{CreateAttachment, GuildId, Http, Member, Mentionable, Message, RoleId, Timestamp};
use poise::serenity_prelude as serenity;
use crate::{DB, log_handle, unwrap_log};
use crate::commands::joke::Joke;
use crate::utils::{CommandResult, Warns};
use crate::utils::MessageData;
use crate::commands::setters::AdminData;
use crate::commands::setters::ForbiddenUserData;
use crate::commands::setters::ForbiddenRoleData;
use crate::commands::setters::set_forbidden_exception::ForbiddenException;
use crate::commands::setters::set_joke_channel::JokeChannelData;
use crate::commands::setters::TimeOutMessageData;
use crate::commands::setters::SetTimeoutTimer;
use crate::commands::setters::WarnMessageData;
use crate::utils::debug::{UnwrapErrors, UnwrapLog};

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
                let data = MessageData::new(new_message.id, audio_url.to_owned(), new_message.author.id, new_message.channel_id, new_message.guild_id, None);
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

    if let Some(joke) = joke { handle_joke(joke, new_message, ctx).await? }

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

    let forbidden_user_data = ForbiddenUserData::new(user_id.into(), guild_id);
    let forbidden_user_id = forbidden_user_data.user_id.parse::<u64>().ok();

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

pub async fn handle_forbidden_role(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: MessageData
) -> CommandResult {
    let author_user_id = new_message.author.id;
    let member = guild_id.member(&ctx.http, author_user_id).await?;

    let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
    let admin_role: Option<AdminData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
    let time_out_timer: Option<SetTimeoutTimer> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
    let time_out_message: Option<WarnMessageData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let time_out_message = time_out_message.unwrap_or(WarnMessageData {
        warn_message: "Por favor no hagas @ a este usuario. Si estás respondiendo un mensaje, considera responder al mensaje sin usar @".to_string(),
        guild_id: GuildId::default(),
    }).warn_message;
    let time_out_timer = time_out_timer.unwrap_log("No hay un tiempo de timeout establecido", CURRENT_MODULE, line!())?.time;

    let admin_role_id = admin_role.unwrap_log("No hay un rol de administrador establecido", CURRENT_MODULE, line!())?.role_id;
    let admin_exception = check_admin_exception(admin_role_id, &member, ctx);

    if admin_exception {
        println!("Admin exception : `sent_message.rs` Line 120");
        return Ok(())
    }

    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;

    if let Some(mut existing_warns) = existing_warns {
        existing_warns.warns += 1;
        existing_warns.add_warn().await?;
        warns = existing_warns;
    } else {
        warns.warns += 1;
        warns.save_to_db().await?;
    }

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("Mensaje eliminado por mencionar a un usuario prohibido de mencionar\nAdvertencia {}/3", warns.warns));
    let http = ctx.http.clone();
    http.send_message(new_message.channel_id, vec![], &message_map).await?;
    let mut member = guild_id.member(&ctx.http, author_user_id).await?;

    if warns.warns >= 3 {
        handle_warns(&mut member, new_message, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    let message = Some("Mensaje eliminado por mencionar a un usuario cuyo rol está prohíbido de mencionar");
    http.delete_message(new_message.channel_id, new_message.id, message).await?;

    Ok(())
}

pub async fn handle_forbidden_user(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: MessageData,
    forbidden_user_id: u64
) -> CommandResult {
    let author_user_id = new_message.author.id;
    if author_user_id == forbidden_user_id {
        return Ok(())
    }

    let forbidden_user_exception = ForbiddenException::have_exception(forbidden_user_id.into()).await?;

    // Si el usuario ha solicitado una excepción o no hay una excepción establecida para este usuario, salir de la función
    if let Some(forbidden_user_exception) = forbidden_user_exception {
        if forbidden_user_exception {
            println!("El usuario ha solicitado una excepción : {}", Location::caller());
            return Ok(())
        }
    }

    let mut member = guild_id.member(&ctx.http, author_user_id).await?;
    let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
    let admin_role: Option<AdminData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let time_out_timer = unwrap_log!(SetTimeoutTimer::get_time_out_timer(guild_id).await?, "No se ha establecido un tiempo de silencio");
    let warn_message = unwrap_log!(WarnMessageData::get_warn_message(guild_id).await?, "No se ha establecido un mensaje de advertencia");
    let time_out_message = unwrap_log!(TimeOutMessageData::get_time_out_message(guild_id).await?, "No se ha establecido un mensaje de silencio");
    let admin_role_id = unwrap_log!(admin_role.clone(), "No se ha establecido un rol de administrador").role_id;
    let admin_role_id_2 = unwrap_log!(admin_role, "No se ha establecido un rol de administrador").role_2_id;

    // Salir de la función si no hay un admin establecido
    if admin_role_id.is_none() {
        log_handle!("No hay un admin establecido: `sent_message.rs` {}", line!());
        return Ok(())
    }

    let admin_exception = check_admin_exception(admin_role_id, &member, ctx);
    let admin_exception_2 = check_admin_exception(admin_role_id_2, &member, ctx);

    if admin_exception || admin_exception_2 {
        println!("Admin exception : {}", Location::caller());
        return Ok(())
    }

    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;

    if let Some(mut existing_warns) = existing_warns {
        existing_warns.warns += 1;
        existing_warns.add_warn().await?;
        warns = existing_warns;
    } else {
        warns.warns += 1;
        warns.save_to_db().await?;
    }

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("{warn_message}\nAdvertencia {}/3", warns.warns));
    let http = ctx.http.clone();
    let attachment = CreateAttachment::path("./assets/sugerencia.png").await?;
    http.send_message(new_message.channel_id, vec![attachment], &message_map).await?;

    message_map.insert("content", String::new());
    let attachment_mobile = CreateAttachment::path("./assets/sugerencia_mobile.png").await?;
    http.send_message(new_message.channel_id, vec![attachment_mobile], &message_map).await?;

    if warns.warns >= 3 {
        handle_warns(&mut member, new_message, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    http.delete_message(new_message.channel_id, new_message.id, None).await?;

    Ok(())
}

async fn handle_warns(
    member: &mut Member,
    new_message: &Message,
    mut message_map: HashMap<&str, String>,
    http: &Arc<Http>,
    mut warns: Warns,
    time_out_timer: i64,
    time_out_message: String,
) -> CommandResult {

    let time = Timestamp::from(Utc::now() + Duration::seconds(time_out_timer));
    member.disable_communication_until_datetime(&http, time).await?;

    message_map.insert("content", format!("{} {}", member.mention(), time_out_message));
    http.send_message(new_message.channel_id, vec![], &message_map).await?;
    warns.reset_warns().await?;

    Ok(())
}

fn check_admin_exception(admin_role_id: Option<String>, member: &Member, ctx: &serenity::Context) -> bool {
    admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles", CURRENT_MODULE, line!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == RoleId::new(admin_role_id.parse::<u64>().unwrap_or_default()))
    })
}

async fn handle_joke(mut joke: Joke, new_message: &Message, ctx: &serenity::Context) -> Result<(), UnwrapErrors> {
    let sql_query = "SELECT * FROM joke_channel WHERE guild_id = $guild_id";
    let joke_channel: Option<JokeChannelData> = DB
        .query(sql_query)
        .bind(("guild_id", &joke.guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let joke_channel = joke_channel
        .unwrap_log("No se ha establecido un canal de broma", CURRENT_MODULE, line!())?
        .channel_id
        .parse::<u64>()?;

    let joke_channel = serenity::all::ChannelId::new(joke_channel);

    let joke_id = joke.target.parse::<u64>()?;
    let joke_status = joke.is_active;

    if joke_status && joke_channel == new_message.channel_id {
        let author_user_id = new_message.author.id;
        if author_user_id == joke_id {
            let mut message_map = HashMap::new();
            message_map.insert("content", " ".to_string());
            let http = ctx.http.clone();
            let attachment = CreateAttachment::path("./assets/joke.gif").await?;
            http.send_message(new_message.channel_id, vec![attachment], &message_map).await?;

            joke.switch(false).await?;
        }
    }

    Ok(())
}