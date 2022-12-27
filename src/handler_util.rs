use serenity::{
    http::Http, 
    model:: {
        application::interaction::{
            application_command::ApplicationCommandInteraction, 
            InteractionResponseType::ChannelMessageWithSource,
        },
        prelude::{ChannelId, Channel},
    },
    prelude::*
};

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


pub async fn response<D>(http: &Http, command: &ApplicationCommandInteraction, msg: D)
where
    D: ToString,
{
    if let Err(why) = command
        .create_interaction_response(http, |response| {
            response
                .kind(ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(msg))
        })
        .await
    {
        println!("{}", why);
    };
}