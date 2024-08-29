use poise::{
    serenity_prelude::{Channel, CreateChannel},
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command)]
pub async fn channel(
    ctx: crate::Context<'_>,
    #[description = "Enter new channel name"] new_name: String,
    channel_id: Channel,
    nsfw_level: bool,
) -> Result<(), Error> {
    let server = ctx.guild_id().unwrap();
    let builder = CreateChannel::new(new_name)
        .kind(poise::serenity_prelude::ChannelType::Text)
        .category(channel_id.id())
        .nsfw(nsfw_level);

    match server.create_channel(&ctx.http(), builder).await {
        Ok(channel) => {
            let message = CreateReply {
                content: Some(format!(
                    "New channel created: {} and it is {} nsfw",
                    channel.name, channel.nsfw
                )),
                ephemeral: Some(true),
                ..Default::default()
            };
            ctx.send(message).await?;
            Ok(())
        }
        Err(why) => Err(why.into()),
    }
}
