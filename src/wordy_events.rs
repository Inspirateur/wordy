use std::collections::HashMap;
use anyhow::anyhow;
use serenity::{
    all::Interaction, async_trait, model:: {
        gateway::Ready,
        guild::Guild, prelude::{Emoji, EmojiId, GuildId, Message},
    }, prelude::*
};
use log::{info, trace, warn};
use crate::wordy::Wordy;


#[async_trait]
impl EventHandler for Wordy {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.to_string();
                if let Err(why) = match command_name.as_str() {
                    "cloud" => self.cloud_command(ctx, command).await,
                    "emojis" => self.emojis_command(ctx, command).await,
                    "info" => self.info_command(ctx, command).await,
                    _ => Err(anyhow!("Unknown command"))
                } {
                    warn!(target: "wordy", "\\{}: {:?}", command_name, why);
                }
            },
            _ => {}
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(target: "wordy", "{} is connected!", ready.user.name);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        self.register_commands(ctx.http.clone(), guild.id).await;
        self.register_guild(ctx.http, guild).await;
    }

    async fn message(&self, _ctx: Context, message: Message) {
        if message.guild_id.is_some() {
            trace!(target: "wordy", "Read a new message from {}", message.author.name);
            self.message(message);
        }
    }

    async fn guild_emojis_update(
        &self,
        _ctx: Context,
        guild_id: GuildId,
        current_state: HashMap<EmojiId, Emoji>,
    ) {
        self.servers_emos.insert(guild_id, current_state.clone());
    }
}
