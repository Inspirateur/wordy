use itertools::Itertools;
use log::{warn, info, trace};
use image::RgbaImage;
use regex::Regex;
use std::{sync::Arc, collections::HashMap};
use anyhow::{Result, bail};
use palette::rgb::Rgb;
use dashmap::DashMap;
use serenity::{
    http::Http, 
    model:: {
        id::GuildId, prelude::{UserId, ChannelId, Guild, Channel, Message, EmojiId, Emoji, Member}
    },
    prelude::*, utils::Color
};
use futures::future::join_all;
use lazy_static::lazy_static;
use wordcloud_rs::{Token, WordCloud, Colors};
use crate::{idiom::{Idioms, tokenize}, discord_emojis::DiscordEmojis, fixed_deque::FixedDeque, emoji_usage::EmojiUsage};

lazy_static! {
    static ref RE_EMO: Regex = Regex::new(r"^<a?:(\w+):(\d+)>$").unwrap();
    static ref RE_TAG: Regex = Regex::new(r"^<@(\d+)>$").unwrap();
    static ref RE_CHAN: Regex = Regex::new(r"^<#(\d+)>$").unwrap();
    static ref RE_ROLE: Regex = Regex::new(r"^<@&(\d+)>$").unwrap();
}

pub struct EmojiRankings {
    pub png: Vec<EmojiUsage>,
    pub gif: Vec<EmojiUsage>
}

fn norm_emo_ranking(emo_ranking: Vec<(Emoji, usize)>) -> Vec<EmojiUsage> {
    let sum = emo_ranking.iter().fold(
        0., |acc, (_, count)| *count as f64 + acc
    );
    emo_ranking.into_iter().map(
        |(emoji_id, count)| EmojiUsage(emoji_id, count as f64/sum)
    ).collect()
}

fn convert_color(color: Color) -> Rgb {
    Rgb::new(
        color.r() as f32/255., 
        color.g() as f32/255., 
        color.b() as f32/255.
    )
}

pub fn register_guild(
    guild: &Guild,
    idioms: Arc<DashMap<GuildId, Idioms<ChannelId, UserId>>>, 
    recents_emos: Arc<DashMap<GuildId, FixedDeque<EmojiId>>>,
    servers_emos: Arc<DashMap<GuildId, HashMap<EmojiId, Emoji>>>,
) -> bool {
    if !idioms.contains_key(&guild.id) {
        info!(target: "wordy", "Registering {} (id {})", guild.name, guild.id);
        idioms.insert(guild.id, Idioms::new());
        recents_emos.insert(guild.id, FixedDeque::new());
        servers_emos.insert(guild.id, guild.emojis.clone());
        true
    } else {
        info!(target: "wordy", "Guild {} (id {}) was already registered", guild.name, guild.id);
        false
    }
}

pub fn read_message(
    guild_id: GuildId,
    message: Message, 
    idioms: Arc<DashMap<GuildId, Idioms<ChannelId, UserId>>>, 
    recents_emos: Arc<DashMap<GuildId, FixedDeque<EmojiId>>>,
    servers_emos: Arc<DashMap<GuildId, HashMap<EmojiId, Emoji>>>,
) {
    if let (
        Some(mut idiom), 
        Some(mut recent_emos),
        Some(server_emos)
    ) = (
        idioms.get_mut(&guild_id), 
        recents_emos.get_mut(&guild_id),
        servers_emos.get(&guild_id)
    ) {
        let tokens = tokenize(message.content);
        tokens
        .iter()
        .filter_map(|token| {
            if let Some(caps) = RE_EMO.captures(token) {
                let emoji_id = EmojiId(
                    caps.get(2).unwrap().as_str().parse::<u64>().unwrap()
                );
                if server_emos.contains_key(&emoji_id) {
                    return Some(emoji_id);
                }
            }
            None
        }).unique()
        .for_each(|emoji_id| recent_emos.push(emoji_id.clone()));
        idiom.update(message.channel_id, message.author.id, tokens);
    } else {
        warn!(target: "wordy", "Guild {} isn't registered yet.", guild_id);
    }
}

pub struct Wordy {
    pub idioms: Arc<DashMap<GuildId, Idioms<ChannelId, UserId>>>,
    pub discord_emos: DiscordEmojis,
    pub recents_emos: Arc<DashMap<GuildId, FixedDeque<EmojiId>>>,
    pub servers_emos: Arc<DashMap<GuildId, HashMap<EmojiId, Emoji>>>,
}

impl Wordy {
    pub fn new() -> Self {
        Self {
            idioms: Arc::new(DashMap::new()),
            discord_emos: DiscordEmojis::new(1000),
            recents_emos: Arc::new(DashMap::new()),
            servers_emos: Arc::new(DashMap::new()),
        }
    }

    pub fn message(&self, message: Message) {
        read_message(
            message.guild_id.unwrap(),
            message,
            self.idioms.clone(), 
            self.recents_emos.clone(), 
            self.servers_emos.clone()
        );
    }

    async fn to_wc_tokens(
        &self, tokens: Vec<(String, f32)>, http: &Arc<Http>
    ) -> Vec<(Token, f32)> {
        join_all(tokens.into_iter().map(|(token, v)| async move {
            if let Some(capts) = RE_EMO.captures(&token) {
                let emo_id = capts.get(2).unwrap().as_str();
                if let Ok(img) = self.discord_emos.get(emo_id).await {
                    (Token::Img(img), v)
                } else {
                    let name = capts.get(1).unwrap().as_str();
                    (Token::Text(name.to_string()), v)
                }
            } else if let Some(capts) = RE_TAG.captures(&token) {
                let user_id = capts.get(1).unwrap().as_str().parse().unwrap();
                if let Ok(member) = http.get_user(user_id).await {
                    (Token::Text(format!("@{}", member.name)), v)
                } else {
                    (Token::Text("@deleted_user".to_string()), v)
                }
            } else if let Some(capts) = RE_CHAN.captures(&token) {
                let chan_id = capts.get(1).unwrap().as_str().parse().unwrap();
                match http.get_channel(chan_id).await {
                    Ok(Channel::Guild(channel)) => (Token::Text(format!("#{}", channel.name)), v),
                    Ok(Channel::Category(channel)) => (Token::Text(format!("#{}", channel.name)), v),
                    _ => (Token::Text("#deleted_channel".to_string()), v)
                }
            } else {
                (Token::Text(token), v)
            }
        }).collect_vec()).await
    }

    pub async fn cloud(&self, ctx: &Context, member: &Member) -> RgbaImage {
        let color = member.colour(&ctx.cache).unwrap_or(Color::from_rgb(255, 255, 255));
        let member_id = member.user.id;
        let tokens = self.idioms.get(&member.guild_id).unwrap().idiom(member_id);
        trace!(target: "wordy", "/cloud: retrieved {} tokens for {}", tokens.len(), member.user.name);
        let wc_tokens = self.to_wc_tokens(tokens, &ctx.http).await;
        WordCloud::new()
        .colors(Colors::BiaisedRainbow { 
            anchor: convert_color(color),
            variance: 50. 
        }).generate(wc_tokens)
    }

    pub fn emojis(&self, guild_id: GuildId) -> Result<EmojiRankings> {
        if let (
            Some(recent_emos),
            Some(server_emos)
        ) = (
            self.recents_emos.get(&guild_id),
            self.servers_emos.get(&guild_id)
        ) {
            let counts = recent_emos.counts();
            let mut png_ranking = Vec::new();
            let mut gif_ranking = Vec::new();
            for (emoji_id, emoji) in server_emos.iter() {
                if emoji.animated {
                    gif_ranking.push((emoji.clone(), *counts.get(emoji_id).unwrap_or(&0)));
                } else {
                    png_ranking.push((emoji.clone(), *counts.get(emoji_id).unwrap_or(&0)));
                }
            }
            png_ranking.sort_by_key(|(_, count)| *count);
            png_ranking.reverse();
            gif_ranking.sort_by_key(|(_, count)| *count);
            gif_ranking.reverse();
            Ok(EmojiRankings { 
                png: norm_emo_ranking(png_ranking), 
                gif: norm_emo_ranking(gif_ranking)
            })
        } else {
            bail!("Guild is not yet registered")
        }
    }
}
