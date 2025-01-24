mod admin;
mod general;
mod models;

use admin::init_sync;
use colored::Colorize;
use dotenv::dotenv;
use general::link;
use models::{RoleErrorResponse, RoleSuccessResponse};
use poise::serenity_prelude::{self as serenity, CacheHttp, Interaction, RoleId};
use std::env;

#[allow(dead_code)]
struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)]
type Context<'a> = poise::Context<'a, Data, Error>;

// This isn't acutally functional, just fun to see.
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

        serenity::FullEvent::InteractionCreate {
            interaction: Interaction::Component(component_interaction),
        } => {
            match component_interaction.data.custom_id.as_str() {
                "sync" => handle_sync_roles(ctx, component_interaction).await?,
                _ => {
                    // Handle other button interactions or do nothing
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_sync_roles(
    ctx: &serenity::Context,
    component_interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    // Acknowledge the button interaction immediately with an ephemeral response
    component_interaction
        .create_response(
            ctx.http(),
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Fetching up-to-date roles...")
                    .ephemeral(true),
            ),
        )
        .await?;

    // Fetch roles from external API
    let author_id = component_interaction.user.id;
    let url = format!("http://localhost:3000/api/discord/{}/sync", author_id);
    let token = std::env::var("DISCORD_TOKEN").expect("No token found in environment variables");

    let response = reqwest::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bot {}", token))
        .send()
        .await?;

    // Ensure the interaction is happening in a guild context
    let guild_id = component_interaction
        .guild_id
        .ok_or_else(|| serenity::Error::Other("Guild ID is required for syncing roles"))?;
    let guild = ctx.http().get_guild(guild_id).await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let result: RoleSuccessResponse = response.json().await?;
            let mut add_roles: Vec<RoleId> = vec![];
            for role in result.roles {
                if let Ok(role_id_num) = role.parse::<u64>() {
                    if let Some(role) = guild.roles.get(&RoleId::new(role_id_num)) {
                        add_roles.push(role.id);
                    }
                }
            }

            let member = ctx
                .http()
                .get_member(guild_id, component_interaction.user.id)
                .await?;

            member.add_roles(ctx.http(), &add_roles).await?;
            // Send a follow-up message indicating completion
            component_interaction
                .create_followup(
                    ctx.http(),
                    serenity::CreateInteractionResponseFollowup::new()
                        .content("Fetched roles. You are now up to date.")
                        .ephemeral(true),
                )
                .await?;
        }
        reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            println!("500: Internal Server Error. Check server logs.");
            // Send a follow-up message indicating completion
            component_interaction
                .create_followup(
                    ctx.http(),
                    serenity::CreateInteractionResponseFollowup::new()
                        .content("Something went wrong... Try again later!")
                        .ephemeral(true),
                )
                .await?;
        }
        _ => {
            let error: RoleErrorResponse = response.json().await?;
            println!("Error: {}", error.error);
            // Send a follow-up message indicating completion
            component_interaction
                .create_followup(
                    ctx.http(),
                    serenity::CreateInteractionResponseFollowup::new()
                        .content("An error has occured, contact server admins for help.")
                        .ephemeral(true),
                )
                .await?;
        }
    }

    Ok(())
}
