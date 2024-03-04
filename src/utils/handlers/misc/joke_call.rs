use std::collections::HashMap;
use serenity::all::{CreateAttachment, Message};
use crate::commands::setters::set_joke::Joke;
use poise::serenity_prelude as serenity;
use crate::commands::setters::set_joke_channel::JokeChannelData;
use crate::utils::misc::debug::UnwrapErrors;

pub async fn handle_joke(mut joke: Joke, new_message: &Message, ctx: &serenity::Context) -> Result<(), UnwrapErrors> {
    let joke_channel = JokeChannelData::get_joke_channel(&joke.guild_id).await?;
    let joke_channel = serenity::all::ChannelId::new(joke_channel);
    let joke_id = joke.target.parse::<u64>()?;
    let joke_status = joke.is_active;
    let author_user_id = new_message.author.id;

    if !joke_status || joke_channel != new_message.channel_id { return Ok(()) }
    if author_user_id != joke_id { return Ok(()) }

    let mut message_map = HashMap::new();
    message_map.insert("content", " ".to_string());
    let http = ctx.http.clone();
    let attachment = CreateAttachment::path("./assets/joke.gif").await?;
    http.send_message(new_message.channel_id, vec![attachment], &message_map).await?;

    joke.switch(false).await?;

    Ok(())
}