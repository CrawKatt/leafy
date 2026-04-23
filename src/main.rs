use std::sync::{Arc, LazyLock};
use std::time::Duration;

use crate::commands::audio::AudioState;
use crate::commands::info::help::get_command_categories;
use crate::handlers::music_events::ready_event;
use chrono::Local;
use handlers::error::handler;
use handlers::events::event_handler;
use lavalink_rs::model::events::Events;
use lavalink_rs::prelude::*;
use poise::serenity_prelude as serenity;
use reqwest::Client as HttpClient;
use serenity::prelude::TypeMapKey;
use songbird::SerenityInit;
use surrealdb::engine::remote::ws::{Client as SurrealClient, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;
use tokio::time::sleep_until;
use tokio::time::Instant;
use utils::debug::UnwrapResult;
use utils::load_commands;
use utils::Data;
use utils::MessageData;

pub static DB: LazyLock<Surreal<SurrealClient>> = LazyLock::new(Surreal::init);

mod commands;
mod utils;
mod handlers;

#[derive(Debug)]
struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[tokio::main]
async fn main() -> UnwrapResult<()> {

    let database_url = dotenvy::var("DATABASE_URL").expect("missing SURREAL_URL");
    let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");
    DB.connect::<Wss>(database_url).await.unwrap_or_else(|why| {
        panic!("Could not connect to database: {why}");
    });

    DB.signin(Root {
        username: "root",
        password: &database_password,
    }).await.expect("Could not sign in");

    // Crear la Base de Datos si no existe
    create_database().await?;

    // Borrar mensajes de la Base de Datos cada 24 horas
    clean_database_loop();

    let token = dotenvy::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::all() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: load_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("leafy".to_lowercase()),
                additional_prefixes: vec![poise::Prefix::Literal("Leafy"), poise::Prefix::Literal(">>")],
                edit_tracker: Some(Arc::from(poise::EditTracker::for_timespan(Duration::from_secs(3600)))),
                ..Default::default()
            },
            on_error: |error| Box::pin(handler(error)),
            event_handler: |ctx, event, framework, _data| Box::pin(event_handler(ctx, event, framework)),
            allowed_mentions: Some(serenity::CreateAllowedMentions::default()
                .all_users(true)
                .replied_user(true)),

            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands
                ).await?;

                let lavalink_host = dotenvy::var("LAVALINK_HOST").expect("missing LAVALINK_HOST");
                let lavalink_port = dotenvy::var("LAVALINK_PORT").expect("missing LAVALINK_PORT");
                let lavalink_password = dotenvy::var("LAVALINK_PASSWORD").expect("missing LAVALINK_PASSWORD");

                let node = NodeBuilder {
                    hostname: format!("{lavalink_host}:{lavalink_port}"),
                    is_ssl: true,
                    events: Events::default(),
                    password: lavalink_password,
                    user_id: UserId(ready.user.id.get()),
                    session_id: None,
                };

                let events = Events {
                    ready: Some(ready_event),
                    ..Default::default()
                };

                println!("Lavalink connecting to {lavalink_host}:{lavalink_port} (SSL: true)...");

                let lavalink: LavalinkClient = LavalinkClient::new(
                    events,
                    vec![node],
                    NodeDistributionStrategy::round_robin()
                ).await;

                println!("Lavalink client initialized.");

                let command_descriptions = get_command_categories(&framework.options().commands);
                let voice_chat_state = Arc::new(Mutex::new(AudioState::Idle));

                Ok(Data {
                    command_descriptions,
                    voice_chat_state,
                    lavalink,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await?;
    
    //let http_client = client.http.clone();

    /*
    tokio::spawn(async move {
        let _ = twitter_monitor(&http_client).await;
    });
    */

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
                Duration::from_hours(24)
            });

            sleep_until(Instant::now() + duration_until_midnight).await;

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

            sleep_until(Instant::now() + Duration::from_hours(24)).await;
        }
    });
}

/// # Crear la tabla de la Base de Datos si no existe
/// - La tabla de configuración de los servidores es `SCHEMAFULL`
/// - Los campos son flexibles y se pueden añadir o eliminar
async fn create_database() -> UnwrapResult<()> {
    DB.query("DEFINE TABLE guild_config SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE true;").await?;
    DB.query("DEFINE FIELD admins ON guild_config FLEXIBLE TYPE option<object>;").await?;
    DB.query("DEFINE FIELD channels ON guild_config FLEXIBLE TYPE option<object>;").await?;
    DB.query("DEFINE FIELD forbidden ON guild_config FLEXIBLE TYPE option<object>;").await?;
    DB.query("DEFINE FIELD messages ON guild_config FLEXIBLE TYPE option<object>;").await?;
    DB.query("DEFINE FIELD time_out ON guild_config FLEXIBLE TYPE option<object>;").await?;

    Ok(())
}