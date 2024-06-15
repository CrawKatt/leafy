use poise::CreateReply;
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton};

use crate::handlers::error::handler;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::lessons;

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
    let button = CreateActionRow::Buttons(vec![
        CreateButton::new("close")
            .label("Cerrar")
            .style(ButtonStyle::Danger)
        ]
    );
    
    let builder = CreateReply::default()
        .content(example)
        .components(vec![button]);
    
    ctx.send(builder).await?;
    
    Ok(())
}