use poise::serenity_prelude as serenity;
use songbird::error::TrackResult;
use serenity::all::{ComponentInteraction, Context, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId};
use crate::location;
use crate::utils::CommandResult;
use crate::utils::debug::UnwrapLog;

pub enum ButtonAction {
    Skip,
    Stop,
    Pause,
    Resume,
    Unknown,
}

impl From<&str> for ButtonAction {
    fn from(item: &str) -> Self {
        match item {
            "skip" => Self::Skip,
            "stop" => Self::Stop,
            "pause" => Self::Pause,
            "resume" => Self::Resume,
            _ => Self::Unknown,
        }
    }
}

pub async fn update_button(ctx: &Context, mc: &ComponentInteraction) -> CommandResult {
    let buttons = generate_row();
    let components = vec![buttons];

    let response = CreateInteractionResponseMessage::new().components(components);
    mc.create_response(&ctx, CreateInteractionResponse::UpdateMessage(response)).await?;

    Ok(())
}

pub async fn handle_action<F>(
    ctx: &Context,
    guild_id: GuildId,
    mc: &ComponentInteraction,
    message: &str,
    action: F
) -> CommandResult
    where
        F: FnOnce(&mut songbird::tracks::TrackQueue) -> TrackResult<()> + Send,
{
    let songbird = songbird::get(ctx).await
        .unwrap_log(location!())?;

    let Some(call) = songbird.get(guild_id) else {
        let response = CreateInteractionResponseMessage::new().content("No estás en un canal de voz");
        mc.create_response(&ctx, CreateInteractionResponse::Message(response)).await?;

        return Ok(());
    };

    let caller = call.lock().await;
    let mut queue = caller.queue().clone();
    action(&mut queue)?;

    if let ButtonAction::Pause | ButtonAction::Resume = ButtonAction::from(mc.data.custom_id.as_str()) {
        drop(caller);
        return Ok(())
    }

    let response = CreateInteractionResponseMessage::new().content(message);
    mc.create_response(&ctx, CreateInteractionResponse::Message(response)).await?;

    drop(caller);

    Ok(())
}

pub async fn handle_and_update<F>(
    ctx: &Context,
    guild_id: GuildId,
    mc: &ComponentInteraction,
    message: &str,
    action: F
) -> CommandResult
    where
        F: FnOnce(&mut songbird::tracks::TrackQueue) -> TrackResult<()> + Send,
{
    handle_action(ctx, guild_id, mc, message, action).await?;
    update_button(ctx, mc).await?;

    Ok(())
}

pub fn generate_row() -> CreateActionRow {
    let pause = CreateButton::new("pause")
        .label("Pausar")
        .emoji('⏸');

    let skip = CreateButton::new("skip")
        .label("Saltar")
        .emoji('⏭');

    let stop = CreateButton::new("stop")
        .label("Detener")
        .emoji('⏹');

    CreateActionRow::Buttons(vec![stop, pause, skip])
}