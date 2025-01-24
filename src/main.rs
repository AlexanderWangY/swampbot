mod admin;
mod general;
mod models;
mod moderator;

use admin::init_sync;
use colored::Colorize;
use dotenv::dotenv;
use general::link;
use models::{RoleErrorResponse, RoleSuccessResponse};
use poise::serenity_prelude::{
    self as serenity, CacheHttp, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, CreateMessage, Interaction, Role, RoleId,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    StatusCode,
};
use std::env;

#[allow(dead_code)]
struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)]
type Context<'a> = poise::Context<'a, Data, Error>;

fn startup_message(str: &str) {
    println!("[{}] {}", "OK".green(), str);
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("No token found in environment variables");
    startup_message("Discord token found");

    let intents = serenity::GatewayIntents::non_privileged();
    startup_message("Initialized gateway intents");

    let data = Data {};
    startup_message("Global variables initialized");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![init_sync(), link()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();
    startup_message("Poise framework built");

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    startup_message("Discord client built");

    startup_message("Starting up discord bot");

    println!(
        "[{}] {} is now {}",
        "INFO".yellow(),
        "Swampbot".purple(),
        "online".green()
    );

    client.unwrap().start().await.unwrap();
}

#[allow(unused_variables, clippy::single_match)]
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Component(component_interaction) = interaction.clone() {
                let message = serenity::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(String::from("Fetching up to date roles..."))
                        .ephemeral(true),
                );

                component_interaction
                    .create_response(ctx.http(), message)
                    .await?;

                // Fetch roles

                let author_id = component_interaction.user.id;

                let secret_key: String =
                    env::var("SECRET_ACCESS_KEY").expect("No token found in environment variables");

                let url = format!("http://localhost:3000/api/discord/{}/sync", author_id);

                let mut headers = HeaderMap::new();

                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bot {}", secret_key))?,
                );

                let client = reqwest::Client::new();

                let response = client.get(&url).headers(headers).send().await?;

                let guild_id = component_interaction.guild_id.unwrap();

                let guild = ctx.http().get_guild(guild_id).await?;

                match response.status() {
                    StatusCode::OK => {
                        let result: RoleSuccessResponse = response.json().await?;

                        if result.roles.is_empty() {
                            println!("No roles");
                        } else {
                            for role in result.roles.iter() {
                                // parse u64 from string
                                let role_id_num = role.parse::<u64>().unwrap();

                                if let Some(role) = guild.roles.get(&RoleId::new(role_id_num)) {
                                    println!("Giving role: {}", role.name);
                                }
                            }
                        }
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        println!("500: Internal Server Error, Please check server logs");
                    }
                    _ => {
                        let result: RoleErrorResponse = response.json().await?;

                        println!("Error: {}", result.error);
                    }
                }

                let done_message = CreateInteractionResponseFollowup::new()
                    .content(String::from("Fetched roles. You are now up to date."))
                    .ephemeral(true);

                component_interaction
                    .create_followup(ctx.http(), done_message)
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
