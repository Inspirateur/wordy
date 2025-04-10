mod idiom;
mod emoji_usage;
mod discord_emojis;
mod discord_util;
mod wordy;
mod wordy_events;
mod wordy_commands;
mod fixed_deque;
use wordy::Wordy;
use env_logger;
use std::fs::read_to_string;
use log::{warn, error, LevelFilter};
use serenity::{
    http::Http,
    model::gateway::GatewayIntents,
    prelude::*,
};
use std::env;


fn get_token(name: &str) -> Option<String> {
    if let Ok(token) = env::var(name) {
        Some(token)
    } else {
        warn!(target: "wordy", "Couldn't find the 'WORDY_TOKEN' environment variable, using token.txt as fallback");
        if let Ok(content) = read_to_string("token.txt") {
            Some(content)
        } else {
            warn!(target: "wordy", "Couldn't access token.txt");
            None
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("wordy", LevelFilter::Trace)
        .init();
    // Configure the client with your Discord bot token in the environment.
    let token = get_token("WORDY_TOKEN").unwrap();
    let http = Http::new(&token);

    // The Application Id is usually the Bot User Id.
    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    // Build our client.
    let mut client = Client::builder(
        token, GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES
    )
        .event_handler(Wordy::new())
        .application_id(bot_id.into())
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!(target: "wordy", "Client error: {:?}", why);
    }
}
