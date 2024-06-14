use serenity::futures;
use futures::{
    Stream,
    StreamExt
};
use crate::utils::Context;

pub async fn args_set_timeout_timer<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["1 Minuto", "5 Minutos", "30 Minutos", "60 Minutos", "1 Semana"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(ToString::to_string)
}