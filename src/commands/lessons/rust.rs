use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::lessons;
use crate::handlers::error::handler;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Lessons",
    on_error = "handler",
    user_cooldown = 10,
)]
pub async fn rust(
    ctx: Context<'_>,
    #[autocomplete = "lessons"]
    #[description = "El concepto de Rust que quieres aprender"] concept: String,
) -> CommandResult {
    let path = format!("./assets/rust-examples/docs/{concept}.md");
    let example = std::fs::read_to_string(path)?;
    ctx.say(example).await?;
    
    Ok(())
}