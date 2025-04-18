use std::collections::HashMap;
use crate::utils::config::{Getter, GuildData};
use crate::utils::CommandResult;
use crate::DB;
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::search::SearchMode;
use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, Http};
use tokio::time::{self, Duration};

// todo: Asignar usuario de Twitter seteandolo en la Base de Datos
pub async fn twitter_monitor(http: &Http) -> CommandResult {
    let mut interval = time::interval(Duration::from_secs(60)); // Verificar cada 60s
    let mut scraper = Scraper::new().await?;

    scraper.login(
        dotenvy::var("TWITTER_USERNAME")?,
        dotenvy::var("TWITTER_PASSWORD")?,
        None,
        None,
    ).await?;

    //let mut last_tweet_id = String::new();
    let mut last_tweet_ids: HashMap<String, String> = HashMap::new();

    loop {
        interval.tick().await;

        let query = "SELECT * FROM guild_config WHERE twitter.channel != NONE AND twitter.user != NONE";
        let guild_configs: Vec<GuildData> = DB
            .query(query)
            .await?
            .take(0)?;

        for config in &guild_configs {
            let twitter = &config.twitter;
            let Some(channel_id) = &twitter.channel else { continue };
            let Some(twitter_user) = &twitter.user else { continue };
            let guild_id = config.clone().id.unwrap().to_id();

            let key = format!("{twitter_user}:{guild_id}");
            let last_tweet_id = last_tweet_ids.entry(key).or_default();

            let search_query = format!("from:{}", twitter_user.trim_start_matches('@'));
            let tweets = scraper.search_tweets(&search_query, 1, SearchMode::Latest, None).await?;
            let Some(latest_tweet) = tweets.tweets.first() else { continue };
            let Some(current_tweet_id) = &latest_tweet.id else { continue };

            if current_tweet_id != last_tweet_id {
                last_tweet_id.clone_from(current_tweet_id);

                let tweet_url = latest_tweet
                    .permanent_url
                    .clone()
                    .map_or_else(|| {
                        format!("https://vxtwitter.com/{}/status/{}", twitter_user.trim_start_matches('@'), current_tweet_id)
                    }, |url| url.replace("twitter.com", "vxtwitter.com"));

                let channel: ChannelId = channel_id.parse()?;
                channel.say(http, format!("Nuevo Tweet de {}: {tweet_url}", twitter_user.trim_matches('@'))).await?;
            }
        }
    }
}