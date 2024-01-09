use once_cell::sync::Lazy;

use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);
mod commands;
mod utils;

use poise::serenity_prelude as serenity;
use crate::commands::ping::ping;
use crate::utils::error::Data;

const OWNER_BOT: u64 = 123456789012345678; // ID del dueño del bot

#[tokio::main]
async fn main() {

    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    client: Default::default()
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}