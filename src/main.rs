use std::sync::Arc;
use surrealdb::Surreal;
use std::thread::sleep;
use std::time::Duration;
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use surrealdb::opt::auth::Root;
use surrealdb::engine::remote::ws::{Client as SurrealClient, Ws};

pub static DB: Lazy<Surreal<SurrealClient>> = Lazy::new(Surreal::init);

mod commands;
mod utils;

use utils::Data;
use utils::MessageData;
use utils::load_commands;
use utils::events::event_handler;
use utils::handlers::error::err_handler;

#[tokio::main]
async fn main() {

    let database_url = dotenvy::var("DATABASE_URL").expect("missing SURREAL_URL");
    let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");
    DB.connect::<Ws>(database_url).await.unwrap_or_else(|why| {
        panic!("Could not connect to database: {why}");
    });

    DB.signin(Root {
        username: "root",
        password: &database_password,
    }).await.expect("Could not sign in");

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

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

fn clean_database_loop() {
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(60 * 60 * 24)); // 24 horas (60 * 60 * 24)

            DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
                log_handle!("Could not use namespace: {why}");
                panic!("Could not use namespace: {why}");
            });

            DB.delete("messages").await.unwrap_or_else(|why| -> Vec<MessageData> {
                log_handle!("Could not delete messages: {why}");
                panic!("Could not delete messages: {why}");
            });
        }
    });
}