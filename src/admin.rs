use poise::{
    serenity_prelude::{Channel, CreateChannel},
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn createchannel(
    ctx: crate::Context<'_>,
    #[description = "Enter new channel name"] new_name: String,
    channel_id: Channel,
) -> Result<(), Error> {
    let server = ctx.guild_id().unwrap();

    let builder = CreateChannel::new(new_name)
        .kind(poise::serenity_prelude::ChannelType::Text)
        .category(channel_id.id());

    match server.create_channel(&ctx.http(), builder).await {
        Ok(_channel) => {
            let message = CreateReply {
                content: Some(String::from("New channel created successfully")),
                ephemeral: Some(true),
                ..Default::default()
            };
            ctx.send(message).await?;
            Ok(())
        }
        Err(_) => {
            let message = CreateReply {
                content: Some(String::from(
                    "Error creating channel, make sure to enter a unique name and a category",
                )),
                ephemeral: Some(true),
                ..Default::default()
            };
            ctx.send(message).await?;
            Ok(())
        }
    }
}

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn deletechannel(
    ctx: crate::Context<'_>,
    #[description = "Enter channel to delete"] channel: Channel,
) -> Result<(), Error> {
    // Retrieve the author information
    let user = ctx.author();

    // If channel is category -> ERROR
    if channel.clone().category().is_some() {
        let message = CreateReply {
            content: Some(String::from(
                "Can't delete categories! Please select a channel.",
            )),
            ephemeral: Some(true),
            ..Default::default()
        };

        if let Err(e) = ctx.send(message).await {
            eprintln!("Error sending message {:?}", e);
            return Err(e.into());
        }

        return Ok(());
    }

    // Attempt to delete the channel
    if let Err(e) = ctx
        .http()
        .delete_channel(channel.id(), Some(&format!("Deleted by {}", user.id)))
        .await
    {
        eprintln!("Error deleting channel: {:?}", e);
        return Err(e.into());
    }

    // Finish with ephemeral reply
    let message = CreateReply {
        content: Some(String::from("Channel deleted successfully")),
        ephemeral: Some(true),
        ..Default::default()
    };

    if let Err(e) = ctx.send(message).await {
        eprintln!("Error sending confirmation message: {:?}", e);
        return Err(e.into());
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn set_ticket_category(
    ctx: crate::Context<'_>,
    #[description = "Category to put ticket channels in"] category: Channel,
) -> Result<(), Error> {
    if category.clone().category().is_none() {
        let message = CreateReply {
            content: Some(String::from(
                "Can't set a channel as a ticket category. Please select a CATEGORY.",
            )),
            ephemeral: Some(true),
            ..Default::default()
        };

        if let Err(e) = ctx.send(message).await {
            eprintln!("Error sending error message: {:?}", e);
            return Err(e.into());
        }

        return Ok(());
    }

    let mut ticket_channel_lock = ctx.data().ticket_channel.lock().await;

    *ticket_channel_lock = Some(category);

    let message = CreateReply {
        content: Some(String::from("Successfully set channel as ticket channel")),
        ephemeral: Some(true),
        ..Default::default()
    };

    if let Err(e) = ctx.send(message).await {
        eprintln!("Error sending error message: {:?}", e);
        return Err(e.into());
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn simulate_ticket(
    ctx: crate::Context<'_>,
    #[description = "Ticket name"] name: String,
) -> Result<(), Error> {
    let server = ctx.guild_id().unwrap();

    let ticket_channel = ctx.data().ticket_channel.lock().await;

    let category_id = ticket_channel.clone().unwrap();

    let builder = CreateChannel::new(name)
        .kind(poise::serenity_prelude::ChannelType::Text)
        .category(category_id);

    match server.create_channel(&ctx.http(), builder).await {
        Ok(_channel) => {
            let message = CreateReply {
                content: Some(String::from("New ticket created successfully")),
                ephemeral: Some(true),
                ..Default::default()
            };
            ctx.send(message).await?;
        }
        Err(_) => {
            let message = CreateReply {
                content: Some(String::from("Something went wrong creating a ticket")),
                ephemeral: Some(true),
                ..Default::default()
            };
            ctx.send(message).await?;
        }
    }

    Ok(())
}
