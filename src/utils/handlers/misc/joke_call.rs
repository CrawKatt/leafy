use std::collections::HashMap;
use serenity::all::{CreateAttachment, Message};
use crate::commands::joke::Joke;
use poise::serenity_prelude as serenity;
use crate::commands::setters::set_joke_channel::JokeChannelData;
use crate::DB;
use crate::utils::debug::{UnwrapErrors, UnwrapLog};

const CURRENT_MODULE: &str = file!();

pub async fn handle_joke(mut joke: Joke, new_message: &Message, ctx: &serenity::Context) -> Result<(), UnwrapErrors> {
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