use itertools::Itertools;
use log::{warn, info, trace};
use image::{write_buffer_with_format, ColorType, ImageOutputFormat};
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
        id::GuildId, prelude::{UserId, ChannelId, Guild}
    },
    prelude::*, utils::Color
};
use wordcloud_rs::{Token, WordCloud, Colors};
use crate::idiom::{Idioms, tokenize};
const READ_PAST: u64 = 10000;

fn convert_color(color: Color) -> Rgb {
    Rgb::new(
        color.r() as f32/255., 
        color.g() as f32/255., 
        color.b() as f32/255.
    )
}

pub struct Handler {
    idioms: Arc<DashMap<GuildId, Idioms<ChannelId, UserId>>>
}

impl Handler {
    pub fn new() -> Self {
        Self {
            idioms: Arc::new(DashMap::new())
        }
    }

    pub fn message(&self, guild_id: GuildId, channel_id: ChannelId, member_id: UserId, message: String) {
        self.idioms.get_mut(&guild_id).unwrap().update(channel_id, member_id, tokenize(message));
    }

    fn to_wc_tokens(&self, tokens: Vec<(String, f32)>) -> Vec<(Token, f32)> {
        // TODO: also convert :emojis: to images 
        tokens.into_iter().map(|(str, v)| (Token::Text(str), v)).collect_vec()
    }

    pub async fn cloud(&self, ctx: Context, command: ApplicationCommandInteraction) {
        if let Some(member) = &command.member {
            let color = member.colour(&ctx.cache).unwrap_or(Color::from_rgb(255, 255, 255));
            if let Some(guild_id) = command.guild_id {
                let member_id = member.user.id;
                let tokens = self.idioms.get(&guild_id).unwrap().idiom(member_id);
                trace!(target: "Wordy", "/cloud: retrieved {} tokens for {}", tokens.len(), member.user.name);
                let wc_tokens = self.to_wc_tokens(tokens);
                let image = WordCloud::new()
                .colors(Colors::DoubleSplitCompl(convert_color(color))).generate(wc_tokens);
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
                    println!("{}", why);
                };
            } else {
                warn!(target: "Wordy", "/cloud: Couldn't get guild");
            }
        } else {
            warn!(target: "Wordy", "/cloud: Couldn't get member");
        }
    }

    pub async fn emojis(&self, ctx: Context, command: ApplicationCommandInteraction) {
        todo!()
    }

    pub async fn activity(&self, ctx: Context, command: ApplicationCommandInteraction) {
        todo!()
    }

    pub async fn register_guild(&self, http: Arc<Http>, guild: Guild) {
        if let Ok(channels) = guild.channels(&http).await {
            if !self.idioms.contains_key(&guild.id) {
                info!(target: "Wordy", "Registering {} (id {})", guild.name, guild.id);
                self.idioms.insert(guild.id, Idioms::new());
                let http = Arc::clone(&http);
                let idioms = Arc::clone(&self.idioms);
                tokio::spawn(async move {
                    for (channel_id, channel) in channels {
                        if let Ok(messages) = channel.messages(
                            &http, |retriever| retriever.limit(READ_PAST)
                        ).await {
                            for message in messages {
                                idioms.get_mut(&guild.id).unwrap().update(
                                    channel_id, message.author.id, tokenize(message.content)
                                );
                            }
                            info!(target: "Wordy", "Read {} past messages in {}/{}", READ_PAST, guild.name, channel.name())
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
                            .name("emojis")
                            .description("Display the most and least used emojis for this server.")
                    }).create_application_command(|command| {
                        command
                            .name("activity")
                            .description("Display stats about the activity of text channels over time.")
                    })
            })
            .await
        {
            println!("Couldn't register slash commmands: {}", why);
        };

    }
}
