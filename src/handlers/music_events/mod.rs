use lavalink_rs::{hook, model::events::{self}, prelude::*};

#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
    client.delete_all_player_contexts().await.unwrap();
    println!("{session_id:?} -> {event:?}");
}