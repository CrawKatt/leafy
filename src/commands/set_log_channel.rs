use serde::{Deserialize, Serialize};
use serenity::all::{Channel, ChannelId, GuildId};
use crate::DB;
use crate::utils::error::{CommandResult, Context};
use crate::utils::autocomplete::autocomplete_log_command;

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildData {
    pub guild_id: GuildId,
    pub log_channel_id: ChannelId,
}

#[poise::command(prefix_command, slash_command)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_log_command"]
    #[description = "The channel to set as the log channel"] channel: Channel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
        panic!("Could not use database: {why}");
    });

    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();

    let data = GuildData {
        guild_id: guild_id,
        log_channel_id: channel_id,
    };

    // Consulta para verificar si el dato ya existe
    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let existing_data: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await.unwrap_or_else(|why| {
        panic!("Could not query database: {why}");
    })
        .take(0).unwrap_or_else(|_| {
        panic!("Could not get guild data");
    });

    match existing_data {
        Some(_) => {
            // Si el dato ya existe, actualízalo
            let sql_query = "UPDATE guilds SET log_channel_id = $log_channel_id WHERE guild_id = $guild_id";
            let _updated: Vec<GuildData> = DB
                .query(sql_query)
                .bind(("log_channel_id", channel_id))
                .bind(("guild_id", guild_id))
                .await.unwrap_or_else(|why| {
                    panic!("Could not update guild: {why}");
                }).take(0).unwrap_or_else(|_| {
                    panic!("Could not get guild data");
                });
        }
        None => {
            // Si el dato no existe, créalo
            let _created: Vec<GuildData> = DB.create("guilds")
                .content(data)
                .await
                .unwrap_or_else(|why| {
                    panic!("Could not create guild: {why}");
                });
        }
    }

    ctx.say(format!("Log channel establecido: <#{}>", channel_id)).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn get_log_channel(
    ctx: Context<'_>,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
        panic!("Could not use database: {why}");
    });

    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await.unwrap_or_else(|why| {
            panic!("Could not query database: {why}");
        })
        .take(0).unwrap_or_else(|why| {
            panic!("Could not get guild data: {why}");
        });

    let log_channel_id = database_info.unwrap().log_channel_id;

    ctx.say(format!("Log channel is <#{}>", log_channel_id)).await?;

    Ok(())
}