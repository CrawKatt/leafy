use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::time::{Duration, sleep};

use serenity::all::{ChannelId, GuildId, TypingStartEvent, UserId};
use tokio::sync::Mutex;
use crate::commands::moderation::set_forbidden_exception::ForbiddenException;

use crate::utils::CommandResult;
use crate::utils::config::load_data;
use crate::utils::debug::IntoUnwrapResult;

type TimerMap<T> = LazyLock<Arc<Mutex<HashMap<T, tokio::task::JoinHandle<()>>>>>;

static TIMERS: TimerMap<UserId> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// # Esta función maneja el evento de inicio de escritura de un usuario
/// - Verifica si el usuario que está escribiendo es el usuario prohibido de tagear `(@)`
/// - Verifica si el canal en el que está escribiendo es el canal de excepciones `(#🌱meica-chat)`
/// - Si se cumplen las condiciones anteriores, activa la excepción de tageo durante 5 minutos
///
/// Nota: Se debe mejorar la forma de obtener el `forbidden_user_id`. Por ahora, solo puede haber
/// un usuario al cual aplicar excepciones.
pub async fn handler(event: &TypingStartEvent) -> CommandResult {
    let user_id = event.user_id;
    let channel_id = event.channel_id;
    let guild_id = event.guild_id.into_result()?;

    // todo: mejorar la forma de obtener el forbidden_user_id, solo puede haber un usuario al cual aplicar excepciones.
    let forbidden_user_id = load_data().forbidden.user.parse::<UserId>()?;
    let exception_channel_id = load_data().channels.exceptions.parse::<ChannelId>()?;

    if user_id == forbidden_user_id && channel_id == exception_channel_id {
        ForbiddenException::manual_switch(user_id, guild_id, true).await?;
        exception_timer(user_id, guild_id, Duration::from_secs(300)).await;
    }

    Ok(())
}

pub async fn exception_timer(user_id: UserId, guild_id: GuildId, duration: Duration) {
    let timers = TIMERS.clone();
    let mut timers_lock = timers.lock().await;

    if let Some(handle) = timers_lock.remove(&user_id) {
        handle.abort();
    }

    let handle = tokio::spawn(async move {
        sleep(duration).await;

        let _ = ForbiddenException::manual_switch(user_id, guild_id, false).await;
    });

    timers_lock.insert(user_id, handle);
    drop(timers_lock);
}