use itertools::Itertools;
use log::{warn, info, trace};
use image::{write_buffer_with_format, ColorType, ImageOutputFormat};
use regex::Regex;
use std::{io::{Cursor, Seek, SeekFrom}, sync::Arc};
use palette::rgb::Rgb;
use dashmap::DashMap;
use serenity::{
    http::Http, 
    model:: {
        application::interaction::{
            application_command::ApplicationCommandInteraction, 
            InteractionResponseType::ChannelMessageWithSource,
        },
        id::GuildId, prelude::{UserId, ChannelId, Guild, Channel}, Timestamp
    },
    prelude::*, utils::Color
};
use futures::future::join_all;
use lazy_static::lazy_static;
use wordcloud_rs::{Token, WordCloud, Colors};
use crate::{idiom::{Idioms, tokenize}, discord_emojis::DiscordEmojis, handler_util::read_past};
const READ_PAST: u64 = 1000;
const DAYS: i64 = 100;

lazy_static! {
    static ref RE_EMO: Regex = Regex::new(r"<a?:(\w*):(\d*)>").unwrap();
    static ref RE_TAG: Regex = Regex::new(r"<@(\d*)>").unwrap();
    static ref RE_CHAN: Regex = Regex::new(r"<#(\d*)>").unwrap();
    static ref RE_ROLE: Regex = Regex::new(r"<@&(\d*)>").unwrap();
}

fn convert_color(color: Color) -> Rgb {
    Rgb::new(
        color.r() as f32/255., 
        color.g() as f32/255., 
        color.b() as f32/255.
    )
}

pub struct Handler {
    idioms: Arc<DashMap<GuildId, Idioms<ChannelId, UserId>>>,
    emojis: DiscordEmojis
}

impl Handler {
    pub fn new() -> Self {
        Self {
            idioms: Arc::new(DashMap::new()),
            emojis: DiscordEmojis::new(1000)
        }
    }

    pub fn message(&self, guild_id: GuildId, channel_id: ChannelId, member_id: UserId, message: String) {
        if let Some(mut idiom) = self.idioms.get_mut(&guild_id) {
            idiom.update(channel_id, member_id, tokenize(message));
        } else {
            warn!(target: "wordy", "Guild {} isn't registered yet.", guild_id);
        }
    }

    async fn to_wc_tokens(
        &self, tokens: Vec<(String, f32)>, http: &Arc<Http>
    ) -> Vec<(Token, f32)> {
        join_all(tokens.into_iter().map(|(str, v)| async move {
            if let Some(capts) = RE_EMO.captures(&str) {
                let emo_id = capts.get(2).unwrap().as_str();
                if let Ok(img) = self.emojis.get(emo_id).await {
                    (Token::Img(img), v)
                } else {
                    let name = capts.get(1).unwrap().as_str();
                    (Token::Text(name.to_string()), v)
                }
            } else if let Some(capts) = RE_TAG.captures(&str) {
                let user_id = capts.get(1).unwrap().as_str().parse().unwrap();
                if let Ok(member) = http.get_user(user_id).await {
                    (Token::Text(format!("@{}", member.name)), v)
                } else {
                    (Token::Text("@deleted_user".to_string()), v)
                }
            } else if let Some(capts) = RE_CHAN.captures(&str) {
                let chan_id = capts.get(1).unwrap().as_str().parse().unwrap();
                match http.get_channel(chan_id).await {
                    Ok(Channel::Guild(channel)) => (Token::Text(format!("#{}", channel.name)), v),
                    Ok(Channel::Category(channel)) => (Token::Text(format!("#{}", channel.name)), v),
                    _ => (Token::Text("#deleted_channel".to_string()), v)
                }
            } else {
                (Token::Text(str), v)
            }
        }).collect_vec()).await
    }

    pub async fn cloud(&self, ctx: Context, command: ApplicationCommandInteraction) {
        if let Some(member) = &command.member {
            let color = member.colour(&ctx.cache).unwrap_or(Color::from_rgb(255, 255, 255));
            if let Some(guild_id) = command.guild_id {
                let member_id = member.user.id;
                let tokens = self.idioms.get(&guild_id).unwrap().idiom(member_id);
                trace!(target: "wordy", "/cloud: retrieved {} tokens for {}", tokens.len(), member.user.name);
                let wc_tokens = self.to_wc_tokens(tokens, &ctx.http).await;
                let image = WordCloud::new()
                    .colors(Colors::BiaisedRainbow { 
                        anchor: convert_color(color),
                        variance: 50. 
                    }).generate(wc_tokens);
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

                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(ChannelMessageWithSource)
                            .interaction_response_data(
                                |message| message.add_file((
                                    img_vec.as_slice(), 
                                    format!("WordCloud_{}.png", member.display_name()).as_str()
                                ))
                            )
                    })
                    .await
                {
                    warn!(target: "wordy", "/cloud: Response failed with `{}`", why);
                };
            } else {
                warn!(target: "wordy", "/cloud: Couldn't get guild");
            }
        } else {
            warn!(target: "wordy", "/cloud: Couldn't get member");
        }
    }

    pub async fn info(&self, ctx: Context, command: ApplicationCommandInteraction) {
        if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(ChannelMessageWithSource)
                .interaction_response_data(
                    |message| message.content(
                        "Made with ❤️ by Inspi#8989\n
                        Repository: <https://github.com/Inspirateur/wordy>"
                    )
                )
        }).await {
            warn!(target: "wordy", "/info: Response failed with `{}`", why);
        };
    }

    pub async fn register_guild(&self, http: Arc<Http>, guild: Guild) {
        // only read messages that are less than 100 days old
        let cutoff_date = Timestamp::from_unix_timestamp(
            Timestamp::now().unix_timestamp() - 3600*24*DAYS
        ).unwrap();
        if let Ok(channels) = guild.channels(&http).await {
            if !self.idioms.contains_key(&guild.id) {
                info!(target: "wordy", "Registering {} (id {})", guild.name, guild.id);
                self.idioms.insert(guild.id, Idioms::new());
                let http = Arc::clone(&http);
                let idioms = Arc::clone(&self.idioms);
                tokio::spawn(async move {
                    for (channel_id, channel) in channels {
                        let messages = read_past(&http, &channel, READ_PAST, cutoff_date).await;
                        let len = messages.len();
                        for message in messages {
                            idioms.get_mut(&guild.id).unwrap().update(
                                channel_id, message.author.id, tokenize(message.content)
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
                        command
                            .name("info")
                            .description("Information about this bot.")
                    })
            }).await {
            println!("Couldn't register slash commmands: {}", why);
        };
    }
}
