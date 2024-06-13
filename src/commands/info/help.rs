use std::collections::HashMap;
use std::fmt::Write;

use poise::{Command, CreateReply};
use serenity::all::{CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};

use crate::utils::{CommandResult, Context, Data, Error};

pub const FOOTER_URL: &str = "https://cdn.discordapp.com/guilds/983473640387518504/users/395631548629516298/avatars/456f92e6e01310c808551557833f13ad.png?size=2048";

// Se debe manejar la interacción con el SelectMenu desde el handler de interacciones
// en `events.rs` utilizando el `custom_id` para identificar el `SelectMenu`.
// Se recomienda ver como ejemplo, el manejo de botones en `handlers/misc/buttons.rs`
// y el manejo de interacciones en `handlers/interactions.rs`.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Info",
    guild_only,
    ephemeral,
    user_cooldown = 10,
    description_localized("es-ES", "Muestra un menú de ayuda con los comandos del Bot"),
    description_localized("en-US", "Shows a help menu with the Bot's commands"),
    description_localized("ja", "ボットのコマンドを表示するヘルプメニュー")
)]
pub async fn help(ctx: Context<'_>) -> CommandResult {
    let select_menu = CreateSelectMenu::new("help_menu", CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("Moderación", "Moderator").emoji('🛠'),
            CreateSelectMenuOption::new("Fun", "Fun").emoji('🎉'),
            CreateSelectMenuOption::new("Información", "Info").emoji('ℹ'),
            CreateSelectMenuOption::new("Audio", "Audio").emoji('🎵'),
        ],
    }).placeholder("Selecciona una categoría de comandos");
    let action_row = CreateActionRow::SelectMenu(select_menu);
    let description = format!(
        "Prefix del bot: `{}`\n\n{}",
        ctx.framework().options.prefix_options.prefix.as_ref().unwrap(), // SAFETY: El prefix siempre está definido
        ctx.framework().user_data.command_descriptions.values().map(String::as_str).collect::<String>()
    );

    let reply = CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::default()
            .title("Help")
            .color(0x0000_ff00)
            .footer(CreateEmbedFooter::new("© CrawKatt").icon_url(FOOTER_URL))
            .description(description)
        )
        .components(vec![action_row.clone()]);

    ctx.send(reply).await?;

    Ok(())
}

pub fn get_command_categories(commands: &[Command<Data, Error>]) -> HashMap<&'static str, String> {
    let mut map = HashMap::new();

    map.insert("Moderator", filter_categories(&mut commands.iter(),"Moderator"));
    map.insert("Fun", filter_categories(&mut commands.iter(), "Fun"));
    map.insert("Info", filter_categories(&mut commands.iter(), "Info"));
    map.insert("Audio", filter_categories(&mut commands.iter(), "Audio"));

    map
}

pub fn filter_categories(
    commands_iter: &mut dyn Iterator<Item = &Command<Data, Error>>,
    selected_category: &str
) -> String {
    let selected_category_lower = selected_category.to_lowercase();
    let categories = commands_iter
        .filter(|cmd| {
            cmd.category
                .as_ref()
                .is_some_and(|name| name.to_lowercase() == selected_category_lower)
        })
        .fold(
            HashMap::new(),
            |mut map: HashMap<Option<&str>, Vec<&str>>, cmd| {
                map.entry(cmd.category.as_deref())
                    .or_default()
                    .push(cmd.name.as_ref());
                map
            },
        );
    
    if categories.is_empty() {
        String::new()
    } else {
        create_description(&categories)
    }
}

fn create_description(categories: &HashMap<Option<&str>, Vec<&str>>) -> String {
    categories
        .iter()
        .fold(String::new(), |mut description, (cat, cmds)| {
            writeln!(
                description,
                "**{}:**\n```\n{}\n```",
                cat.unwrap_or("None"),
                cmds.join("\n")
            )
            .unwrap();
            description
        })
}