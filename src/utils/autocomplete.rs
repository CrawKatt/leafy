use serenity::futures;
use futures::{
    Stream,
    StreamExt
};
use crate::utils::Context;

/// # Panic if docs folder not found
pub async fn lessons<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let Ok(docs) = std::fs::read_dir("./assets/rust-examples/docs") else {
        panic!("Docs not found required!!!");
    };

    let mut files = vec![];
    for entry in docs {
        let entry = entry.unwrap();
        let filename = entry.file_name().into_string().unwrap();
        let name = filename.split('.').next().unwrap();
        files.push(name.to_string());
    }

    futures::stream::iter(files)
        .filter(move |data: &String| futures::future::ready(data.starts_with(partial)))
}