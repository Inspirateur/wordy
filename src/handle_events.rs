use serenity::{
    model:: {
        application::interaction::{
            Interaction,
        },
        gateway::Ready,
        guild::Guild, prelude::Message,
    },
    async_trait,
    prelude::*
};
use log::{info, trace};
use crate::handler_util::{response, is_writable};
use crate::handler::Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                // only answer if the bot has access to the channel
                if is_writable(&ctx, command.channel_id).await {
                    match command.data.name.as_str() {
                        "cloud" => self.cloud(ctx, command).await,
                        "info" => self.info(ctx, command).await,
                        _ => {}
                    };
                } else {
                    response(
                        &ctx.http,
                        &command,
                        "Sorry, I only answer to commands in the channels that I can write to.",
                    ).await;
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(target: "wordy", "{} is connected!", ready.user.name);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        self.register_commands(ctx.http.clone(), guild.id).await;
        self.register_guild(ctx.http, guild).await;
    }

    async fn message(&self, _ctx: Context, message: Message) {
        if let Some(guild_id) = message.guild_id {
            trace!(target: "wordy", "Read a new message from {}", message.author.name);
            self.message(guild_id, message.channel_id, message.author.id, message.content);
        }
    }
}
