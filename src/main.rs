mod admin;
mod general;
mod moderator;

use admin::{createchannel, deletechannel, set_ticket_category, simulate_ticket};
use colored::Colorize;
use dotenv::dotenv;
use general::link;
use poise::serenity_prelude::{self as serenity, futures::lock::Mutex, Channel};
use std::{env, sync::Arc};

#[allow(dead_code)]
struct Data {
    ticket_channel: Arc<Mutex<Option<Channel>>>,
    current_ticket_id: Arc<Mutex<Option<i32>>>,
}

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

    let data = Data {
        ticket_channel: Arc::new(Mutex::new(None)),
        current_ticket_id: Arc::new(Mutex::new(Some(1))),
    };
    startup_message("Global variables initialized");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                createchannel(),
                deletechannel(),
                set_ticket_category(),
                simulate_ticket(),
                link(),
            ],
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
        _ => {}
    }
    Ok(())
}
