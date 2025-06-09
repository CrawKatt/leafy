use lavalink_rs::{hook, model::events::{self}, prelude::*};

#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
    client.delete_all_player_contexts().await.unwrap();
    println!("{:?} -> {:?}", session_id, event);
}

#[hook]
pub async fn track_start(_: LavalinkClient, _: String, _: &events::TrackStart) {
    println!("Track start");
}

#[hook]
pub async fn raw_event(_: LavalinkClient, session_id: String, event: &serde_json::Value) {
    if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
        println!("{:?} -> {:?}", session_id, event);
    }
}