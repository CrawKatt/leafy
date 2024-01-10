use std::env;
use once_cell::sync::Lazy;

use surrealdb::engine::local::{Db, File};
use surrealdb::Surreal;
pub static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

mod commands;
mod utils;
mod events;

use poise::serenity_prelude as serenity;

use crate::commands::ping::ping;
use crate::events::event_handler;
use crate::utils::error::{Data, err_handler};

#[tokio::main]
async fn main() {
    let database_path = env::current_dir().unwrap_or_default().join("database/");
    DB.connect::<File>(database_path).await.unwrap_or_else(|why| {
        panic!("Could not connect to database: {why}");
    });

    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            on_error: |error| Box::pin(err_handler(error)),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: Default::default(),
                    client: Default::default(),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

