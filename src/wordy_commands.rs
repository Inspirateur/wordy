use crate::{
    discord_util::{read_past, Attachment, Bot},
    wordy::{read_message, register_guild, Wordy},
};
use anyhow::{anyhow, bail, Result};
use image::{write_buffer_with_format, ColorType, ImageOutputFormat};
use itertools::Itertools;
use log::{info, trace, warn};
use serenity::{
    http::Http,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{Emoji, Guild, GuildId},
        Timestamp,
    },
    prelude::Context,
};
use std::{
    io::{Cursor, Seek, SeekFrom},
    sync::Arc,
};
const READ_PAST: u64 = 1000;
const DAYS: i64 = 100;
const TOP_EMO: usize = 5;
const BOTTOM_EMO: usize = 15;
fn emo_entry_mst(rank: usize, freq: f64, emos: Vec<Emoji>) -> String {
    // limit to 20 emojis because the message gets too long otherwise
    let ellipsis = if emos.len() > BOTTOM_EMO { "… " } else { "" };
    let emo_str = emos.into_iter().take(BOTTOM_EMO).join("");
    format!("{}. {}{}: {:.0}%", rank, emo_str, ellipsis, freq * 100.0)
}

fn emo_ranking_msg(emo_ranking: Vec<(Emoji, f64)>) -> String {
    if emo_ranking.len() == 0 {
        return "No entries :(".to_string();
    }
    let mut grouped_ranking = Vec::new();
    for (freq, emos) in &emo_ranking
        .into_iter()
        .group_by(|(_, freq)| (freq * 100.0).round() / 100.)
    {
        grouped_ranking.push((freq, emos.map(|(emoji_id, _)| emoji_id).collect_vec()));
    }
    let toplen = if grouped_ranking.len() <= TOP_EMO + 1 {
        TOP_EMO + 1
    } else {
        TOP_EMO
    };
    let top = grouped_ranking
        .iter()
        .cloned()
        .enumerate()
        .take(toplen)
        .map(|(i, (freq, emos))| emo_entry_mst(i + 1, freq, emos))
        .join("\n");
    if toplen == TOP_EMO + 1 {
        top
    } else {
        let (freq, emos) = grouped_ranking.pop().unwrap();
        let bottom = emo_entry_mst(grouped_ranking.len(), freq, emos);
        format!("{}\n...\n{}", top, bottom)
    }
}

impl Wordy {
    pub async fn cloud_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) -> Result<()> {
        if command.guild_id.is_none() {
            bail!("Command wasn't invoked in a Guild.");
        }
        let member = command
            .member
            .as_ref()
            .ok_or(anyhow!("Couldn't get member."))?;
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
        ctx.http
            .answer(
                &command,
                "",
                vec![Attachment {
                    file: img_vec,
                    filename: format!("WordCloud_{}.png", member.display_name()),
                }],
            )
            .await
    }

    pub async fn emojis_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) -> Result<()> {
        let guild_id = command
            .guild_id
            .as_ref()
            .ok_or(anyhow!("Couldn't get member."))?;
        let emoji_rankings = self.emojis(*guild_id)?;
        let png_msg = "Static emoji ranking:\n".to_string() + &emo_ranking_msg(emoji_rankings.png);
        let gif_msg =
            "Animated emoji ranking:\n".to_string() + &emo_ranking_msg(emoji_rankings.gif);
        ctx.http.answer(&command, &png_msg, vec![]).await?;
        ctx.http.followup(&command, &gif_msg, vec![]).await
    }

    pub async fn info_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) -> Result<()> {
        ctx.http
            .answer(
                &command,
                "Made with ❤️ by Inspi#8989\n
                Repository: <https://github.com/Inspirateur/wordy>",
                vec![],
            )
            .await
    }

    pub async fn register_commands(&self, http: Arc<Http>, guild_id: GuildId) {
        trace!(target: "wordy", "Registering slash commands for Guild {}", guild_id);
        if let Err(why) = GuildId::set_application_commands(&guild_id, http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("cloud")
                        .description("Discover the word cloud that defines you !")
                })
                .create_application_command(|command| {
                    command
                        .name("emojis")
                        .description("Recent emoji usage stats.")
                })
                .create_application_command(|command| {
                    command
                        .name("info")
                        .description("Information about this bot.")
                })
        })
        .await
        {
            warn!(target: "wordy", "Couldn't register slash commmands: {}", why);
        };
    }

    pub async fn register_guild(&self, http: Arc<Http>, guild: Guild) {
        // only read messages that are less than 100 days old
        let cutoff_date =
            Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() - 3600 * 24 * DAYS)
                .unwrap();
        if let Ok(channels) = guild.channels(&http).await {
            if !register_guild(
                &guild,
                self.idioms.clone(),
                self.recents_emos.clone(),
                self.servers_emos.clone(),
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
                        read_message(
                            guild.id,
                            message,
                            idioms.clone(),
                            recents_emos.clone(),
                            servers_emos.clone(),
                        );
                    }
                    if len > 0 {
                        info!(target: "wordy", "Read {} past messages in {}/{}", len, guild.name, channel.name())
                    }
                }
            });
        }
    }
}
