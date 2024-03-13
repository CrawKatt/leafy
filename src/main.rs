use std::env;
use std::sync::Arc;
use surrealdb::Surreal;
use std::time::Duration;
use tokio::time::Instant;
use chrono::Local;
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use surrealdb::engine::local::{Db, File};
use tokio::time::sleep_until;

pub static DB: Lazy<Surreal<Db>> = Lazy::new(Surreal::init);

mod commands;
mod utils;
mod test;

use utils::Data;
use utils::MessageData;
use utils::load_commands;
use utils::events::event_handler;
use utils::handlers::error::err_handler;
use utils::misc::debug::UnwrapResult;

#[tokio::main]
async fn main() -> UnwrapResult<()> {

    let database_path = env::current_dir().unwrap_or_default().join("database/");
    DB.connect::<File>(database_path).await.unwrap_or_else(|why| {
        panic!("Could not connect to database: {why}");
    });

    // Borrar mensajes de la Base de Datos cada 24 horas
    clean_database_loop();

    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: load_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("$".into()),
                edit_tracker: Some(Arc::from(poise::EditTracker::for_timespan(Duration::from_secs(3600)))),
                ..Default::default()
            },
            on_error: |error| Box::pin(err_handler(error)),
            event_handler: |ctx, event, framework, _data| {
                Box::pin(event_handler(ctx, event, framework))
            },
            allowed_mentions: Some(serenity::CreateAllowedMentions::default()
                .all_users(true)
                .replied_user(true)),

            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    poise_mentions: String::default(),
                    client: Client::default(),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}

#[allow(deprecated)]
fn clean_database_loop() {
    tokio::spawn(async {
        loop {
            let now = Local::now();
            let midnight = now + chrono::Duration::days(1);
            let midnight = midnight.date().and_hms(0, 0, 0);
            let duration_until_midnight = (midnight - now).to_std().unwrap_or_else(|why| {
                log_handle!("Could not get duration until midnight: {why}");
                Duration::from_secs(60 * 60 * 24)
            });

            sleep_until(Instant::now() + duration_until_midnight).await; // 24 horas (60 * 60 * 24)

            DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
                log_handle!("Could not use namespace: {why}");
                panic!("Could not use namespace: {why}");
            });

            DB.delete("messages").await.unwrap_or_else(|why| -> Vec<MessageData> {
                log_handle!("Could not delete messages: {why}");
                panic!("Could not delete messages: {why}");
            });

            DB.delete("audio").await.unwrap_or_else(|why| -> Vec<MessageData> {
                log_handle!("Could not delete audio: {why}");
                panic!("Could not delete audio: {why}");
            });

            sleep_until(Instant::now() + Duration::from_secs(60 * 60 * 24)).await;
        }
    });
}