use std::{io::{Cursor, Seek, SeekFrom}, sync::Arc};
use log::{trace, info};
use image::{write_buffer_with_format, ColorType, ImageOutputFormat};
use anyhow::{Result, bail, anyhow};
use serenity::{http::Http, model::{
    prelude::{GuildId, Guild}, 
    application::interaction::application_command::ApplicationCommandInteraction, 
    Timestamp
}, prelude::Context};
use crate::{wordy::{Wordy, register_guild, read_message}, discord_util::{read_past, Bot, Attachment}, emoji_usage::emo_ranking_msg};
const READ_PAST: u64 = 1000;
const DAYS: i64 = 100;

impl Wordy {
    pub async fn cloud_command(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        if command.guild_id.is_none() {
            bail!("Command wasn't invoked in a Guild.");
        }
        let member = command.member.as_ref().ok_or(anyhow!("Couldn't get member."))?;
        let image = self.cloud(&ctx, &member).await;
        let mut img_file = Cursor::new(Vec::new());
        write_buffer_with_format(
            &mut img_file,
            image.as_raw(),
            image.width(),
            image.height(),
            ColorType::Rgba8,
            ImageOutputFormat::Png,
        )
        .unwrap();
        img_file.seek(SeekFrom::Start(0)).unwrap();
        let img_vec = img_file.into_inner();
        ctx.http.answer(
            &command,
            "",
            vec![Attachment {
                file: img_vec, 
                filename: format!("WordCloud_{}.png", member.display_name())
            }]
        ).await
    }

    pub async fn emojis_command(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        let guild_id = command.guild_id.as_ref().ok_or(anyhow!("Couldn't get member."))?;
        let emoji_rankings = self.emojis(*guild_id)?;
        let png_msg = "Static emoji ranking:\n".to_string() 
            + &emo_ranking_msg(emoji_rankings.png);
        let gif_msg = "Animated emoji ranking:\n".to_string() 
            + &emo_ranking_msg(emoji_rankings.gif);
        ctx.http.answer(&command, &png_msg, vec![]).await?;
        ctx.http.followup(&command, &gif_msg, vec![]).await
    }

    pub async fn info_command(&self, ctx: Context, command: ApplicationCommandInteraction) -> Result<()> {
        ctx.http.answer(
            &command,
            "Made with ❤️ by Inspi#8989\n
            Repository: <https://github.com/Inspirateur/wordy>",
            vec![]
        ).await
    }

    pub async fn register_commands(&self, http: Arc<Http>, guild_id: GuildId) {
        trace!("Registering slash commands for Guild {}", guild_id);
        if let Err(why) =
            GuildId::set_application_commands(&guild_id, http, |commands| {
                commands
                    .create_application_command(|command| {
                        command.name("cloud").description(
                            "Discover the word cloud that defines you !",
                        )
                    })
                    .create_application_command(|command| {
                        command.name("emojis").description(
                            "Recent emoji usage stats.",
                        )
                    })
                    .create_application_command(|command| {
                        command
                            .name("info")
                            .description("Information about this bot.")
                    })
            }).await {
            println!("Couldn't register slash commmands: {}", why);
        };
    }

    pub async fn register_guild(&self, http: Arc<Http>, guild: Guild) {
        // only read messages that are less than 100 days old
        let cutoff_date = Timestamp::from_unix_timestamp(
            Timestamp::now().unix_timestamp() - 3600*24*DAYS
        ).unwrap();
        if let Ok(channels) = guild.channels(&http).await {
            if !register_guild(
                &guild, 
                self.idioms.clone(), 
                self.recents_emos.clone(), 
                self.servers_emos.clone()
            ) {
                return;
            }
            let http = Arc::clone(&http);
            let idioms = Arc::clone(&self.idioms);
            let recents_emos = Arc::clone(&self.recents_emos);
            let servers_emos = Arc::clone(&self.servers_emos);
            tokio::spawn(async move {
                for (_channel_id, channel) in channels {
                    let messages = read_past(&http, &channel, READ_PAST, cutoff_date).await;
                    let len = messages.len();
                    for message in messages {
                        read_message(guild.id, message, idioms.clone(), recents_emos.clone(), servers_emos.clone());
                    }
                    if len > 0 {
                        info!(target: "wordy", "Read {} past messages in {}/{}", len, guild.name, channel.name())
                    }
                }
            });
        }
    }
}