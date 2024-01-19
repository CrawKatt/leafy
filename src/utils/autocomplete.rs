use serenity::futures;
use futures::{
    Stream,
    StreamExt
};
use crate::utils::Context;

pub async fn args_log_command<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["channel"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}
pub async fn args_set_role<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["role"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}
pub async fn args_set_forbidden_user<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["channel"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}

pub async fn args_set_forbidden_role<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["role"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}

pub async fn args_set_timeout_timer<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(["time"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(std::string::ToString::to_string)
}