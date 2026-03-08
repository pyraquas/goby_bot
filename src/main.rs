mod magic;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.channel_id.name(&ctx).await.unwrap().as_str() {
            "bot-sandbox" => {
                if let Some(response) = magic::message::handle_message(&ctx, &msg).await {
                    if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            _ => {
                if let Err(why) = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        "I'm sorry but no command available in this channel",
                    )
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Error starting client: {:?}", why);
    }
}
