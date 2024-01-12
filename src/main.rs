use std::env;
use std::sync::Arc;
use surrealdb::Surreal;
use std::thread::sleep;
use std::time::Duration;
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use surrealdb::engine::local::{Db, File};

pub static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

mod commands;
mod utils;
mod events;

use crate::commands::ping::ping;
use crate::commands::set_forbidden_user::set_forbidden_user;
use crate::commands::set_timeout_role::set_time_out_role;
use crate::events::{Data, event_handler};
use crate::commands::set_log_channel::{get_log_channel, set_log_channel};
use crate::utils::handlers::error::err_handler;
use crate::utils::MessageData;

#[tokio::main]
async fn main() {
    let database_path = env::current_dir().unwrap_or_default().join("database/");
    DB.connect::<File>(database_path).await.unwrap_or_else(|why| {
        panic!("Could not connect to database: {why}");
    });

    // Borrar mensajes de la Base de Datos cada 24 horas
    clean_database_loop().await;

    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                set_log_channel(),
                get_log_channel(),
                set_time_out_role(),
                set_forbidden_user(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                edit_tracker: Some(Arc::from(poise::EditTracker::for_timespan(Duration::from_secs(3600)))),
                ..Default::default()
            },
            on_error: |error| Box::pin(err_handler(error)),
            event_handler: |ctx, event, framework, _data| {
                Box::pin(event_handler(ctx, event, framework))
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

async fn clean_database_loop() {
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(60 * 60 * 24)); // 24 horas (60 * 60 * 24)

            DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
                panic!("Could not use namespace: {why}");
            });

            DB.delete("messages").await.unwrap_or_else(|why| -> Vec<MessageData> {
                panic!("Could not delete messages: {why}");
            });
        }
    });
}