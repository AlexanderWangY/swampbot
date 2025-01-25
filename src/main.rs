mod admin;
mod general;
mod models;

use admin::{init_sync, set_image_channel};
use colored::Colorize;
use dotenv::dotenv;
use general::link;
use models::{RoleErrorResponse, RoleSuccessResponse};
use poise::serenity_prelude::{
    self as serenity, futures::lock::Mutex, CacheHttp, ChannelId, Interaction, Message, RoleId,
};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct PrintImageRequest {
    image_url: String,
}

#[allow(dead_code)]
struct Data {
    image_channel: Mutex<Option<ChannelId>>,
}

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

    startup_message("Global variables initialized");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![init_sync(), link(), set_image_channel()],
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
                Ok(Data {
                    image_channel: Mutex::new(None),
                })
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

        serenity::FullEvent::Message { new_message } => {
            if let Some(image_channel_id) = *data.image_channel.lock().await {
                if image_channel_id == new_message.channel_id {
                    let message = ctx
                        .http()
                        .get_message(image_channel_id, new_message.id)
                        .await?;

                    if let Some(image) = find_first_image(&message) {
                        let payload = PrintImageRequest {
                            image_url: image.url,
                        };

                        println!("Print image right now... Give us a few moments...");
                        let res = reqwest::Client::new()
                            .post("http://0.0.0.0:8080/print-image")
                            .json(&payload)
                            .send()
                            .await;

                        match res {
                            Ok(response) => {
                                if response.status().is_success() {
                                    println!("Printed image!");
                                } else {
                                    eprintln!("Something went wrong: {:?}", response.status());
                                }
                            }
                            Err(e) => eprintln!("Request failed: {}", e),
                        }
                    }
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
    let url = format!("https://app.swamphacks.com/api/discord/{}/sync", author_id);
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

struct Image {
    url: String,
}

fn find_first_image(message: &Message) -> Option<Image> {
    message
        .attachments
        .iter()
        .find(|attachment| {
            attachment
                .content_type
                .as_ref()
                .map_or(false, |ct| ct.starts_with("image/"))
        })
        .and_then(|attachment| {
            attachment.height.map(|_| Image {
                url: attachment.url.clone(),
            })
        })
}
