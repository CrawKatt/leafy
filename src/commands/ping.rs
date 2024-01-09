//use crate::utils::error::{Context, CommandResult};

//use std::time::Instant;
//use ::serenity::builder::{CreateAllowedMentions, CreateEmbed};
//use poise::CreateReply;
use crate::utils::error::{CommandResult, Context};

/// Pong!
#[poise::command(slash_command)]
pub async fn ping(
    ctx: Context<'_>,
) -> CommandResult {
    ctx.say("Pong!").await?;

    Ok(())
}

/*
use chrono::{DateTime, Utc};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::channel::Message;
*/
/*
#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let created = DateTime::<Utc>::from_utc(msg.timestamp.naive_utc(), Utc);
    let response_time = now.signed_duration_since(created).num_milliseconds();

    let color;
    let response_time_string = response_time.to_string();

    if response_time < 100 {
        color = '🟢';
    } else if response_time >= 200 && response_time < 500 {
        color = '🟡';
    } else {
        color = '🔴';
    }

    msg.reply(&ctx.http, format!("Pong! {}ms {}", response_time_string, color)).await?;

    Ok(())
}
*/
/*
#[no_mangle]
async fn ping_inner(ctx: Context<'_>) -> CommandResult {

}
*/