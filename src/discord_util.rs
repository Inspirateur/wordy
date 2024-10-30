use std::sync::Arc;
use serenity::{
    all::{CommandInteraction, CreateAttachment, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, GetMessages}, async_trait, http::Http, model:: {
        prelude::{Channel, ChannelId, GuildChannel, Message}, Timestamp,
    }, prelude::*
};
use anyhow::{Result, Context as ContextErr};
const DISCORD_READ_LIMIT: u64 = 100;

type Command = CommandInteraction;
pub struct Attachment { pub file: Vec<u8>, pub filename: String }


#[async_trait]
pub trait Bot {
    async fn answer(&self, command: &Command, content: &str, files: Vec<Attachment>) -> Result<()>;

    async fn followup(&self, command: &Command, content: &str, files: Vec<Attachment>) -> Result<()>;
}

#[async_trait]
impl Bot for Http {
    async fn answer(&self, command: &Command, content: &str, files: Vec<Attachment>) -> Result<()> {
        (
            command
            .create_response(self, 
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .add_files(files.into_iter().map(|a| CreateAttachment::bytes(a.file, a.filename)))
                )).await
        ).context("Command create response failed")
    }

    async fn followup(&self, command: &Command, content: &str, files: Vec<Attachment>) -> Result<()> {
        (
            command
            .create_followup(self, 
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .add_files(files.into_iter().map(|a| CreateAttachment::bytes(a.file, a.filename)))
            ).await
        ).context("Command create followup failed")?;
        Ok(())
    }
}

pub async fn is_writable(ctx: &Context, channel_id: ChannelId) -> bool {
    if let Ok(Channel::Guild(channel)) = channel_id.to_channel(&ctx.http).await {
        if let Ok(me) = ctx.http.get_current_user().await {
            if let Ok(perms) = channel.permissions_for_user(&ctx.cache, me.id) {
                return perms.send_messages();
            }
        }
    }
    false
}


pub async fn read_past(http: &Arc<Http>, channel: &GuildChannel, limit: u64, cutoff_date: Timestamp) -> Vec<Message> {
    // Discord's API has a limit of 100 for retrieving past messages, 
    // so we just call it iteratively to get any amount we want, 
    // each time starting on the last message we read
    let mut res: Vec<Message> = Vec::new();
    let mut remaining = limit;
    'outer: while remaining > 0 {
        if let Ok(messages) = channel.messages(
            &http, 
            if let Some(message) = res.last() {
                GetMessages::new().before(message.id).limit(remaining.min(DISCORD_READ_LIMIT) as u8)
            } else {
                GetMessages::new().limit(remaining.min(DISCORD_READ_LIMIT) as u8)
            }
        ).await {
            if messages.len() == 0 {
                break;
            }
            for message in messages {
                if message.timestamp < cutoff_date {
                    break 'outer;
                }
                res.push(message);
            }
            // res.len() should never be bigger than limit unless the API is returning us more message than we asked
            remaining = limit - (res.len() as u64).min(limit); 
        } else {
            break;
        }
    }
    res
}