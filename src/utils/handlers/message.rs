use serenity::all::Message;
use poise::serenity_prelude as serenity;
use crate::DB;
use crate::events::Error;
use crate::utils::MessageData;
use crate::commands::set_timeout_role::RoleData;
use crate::commands::set_forbidden_user::ForbiddenUserData;
pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> Result<(), Error> {
    if new_message.author.bot {
        return Ok(());
    }

    // Obtener el canal de logs de la base de datos
    let data = MessageData::new(
        new_message.id,
        new_message.content.clone(),
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
    );

    // variable que busca la mención en el menssage_content si existe
    let message_content = new_message.content.clone();
    // Si el mensaje no contiene una mención, guardar el mensaje en la base de datos
    // (NECESARIO PARA EVITAR EL PANIC)
    if !message_content.contains("<@") {
        let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
        return Ok(());
    }

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    // Obtener el usuario prohibido de mencionar desde la base de datos
    let sql_query = "SELECT * FROM forbidden_users WHERE user_id = $user_id";
    let forbidden_user: Option<ForbiddenUserData> = DB
        .query(sql_query)
        .bind(("user_id", user_id)) // pasar el valor
        .await?
        .take(0)?;

    let forbidden_user_id = forbidden_user.unwrap_or_default().user.id;
    // Si el usuario prohibido de mencionar es mencionado, silenciar al autor del mensaje
    if new_message.mentions_user_id(forbidden_user_id) {
        let author_user_id = new_message.author.id;
        let Some(guild_id) = new_message.guild_id else {
            println!("Failed to get guild id");
            return Ok(());
        };

        let Ok(member) = guild_id.member(&ctx.http, author_user_id).await else {
            println!("Failed to get member");
            return Ok(());
        };

        let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
        let time_out_role: Option<RoleData> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id)) // pasar el valor
            .await?
            .take(0)?;

        let time_out_role_id = time_out_role.unwrap_or_default().role_id;
        let Ok(_) = member.add_role(&ctx.http, time_out_role_id).await else {
            println!("Failed to add muted role to member");
            return Ok(());
        };
        println!("Silenciado");

        let http = ctx.http.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            match member.remove_role(&http, time_out_role_id).await {
                Ok(_) => println!("Desilenciado"),
                Err(err) => println!("Failed to remove muted role from member: {:?}", err),
            }
        });

        return Ok(());
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    println!("Message created: {:?}", _created);

    Ok(())
}