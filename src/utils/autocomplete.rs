use serenity::futures;
use crate::utils::error::Context;
use futures::{
    Stream,
    StreamExt
};

pub async fn autocomplete_log_command<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["channel"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}