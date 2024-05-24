use std::collections::HashMap;
use std::sync::Arc;
use chrono::{Duration, Utc};
use serenity::all::{Http, Member, Mentionable, Message, Timestamp};
use crate::utils::{CommandResult, Warns};

pub async fn handle_warn_system(
    member: &mut Member,
    new_message: &Message,
    mut message_map: HashMap<&str, String>,
    http: &Arc<Http>,
    mut warns: Warns,
    time_out_timer: i64,
    time_out_message: String,
) -> CommandResult {

    let try_seconds = Duration::try_seconds(time_out_timer);
    let time = Timestamp::from(Utc::now() + try_seconds.unwrap_or_default());
    member.disable_communication_until_datetime(&http, time).await?;

    message_map.insert("content", format!("{} {}", member.mention(), time_out_message));
    http.send_message(new_message.channel_id, vec![], &message_map).await?;
    warns.reset_warns().await?;

    Ok(())
}
