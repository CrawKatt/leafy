use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{id::GuildId, voice::VoiceState}
};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let self_user_id = {
            let self_user = ctx.cache.current_user();
            println!("Bot iniciado como {}", self_user.name);
            self_user.id
        };
    }
}