mod archipelago;
mod magic;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !msg.content.starts_with('!') {
            return;
        }

        if msg.author.name == "sticks548" {
            if let Err(why) = msg
                .channel_id
                .say(
                    &ctx.http,
                    "Hey Sticks, you are not allowed to use commands, sorry :p",
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }
            return;
        }
        if msg.content.starts_with("!hello") {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Salut {} !", msg.author.display_name()))
                .await
            {
                println!("Error sending message: {:?}", why);
            }
            return;
        }
        match msg.channel_id.name(&ctx).await.unwrap().as_str() {
            "archi-orga" | "archi-general" => {
                if let Some(response) = archipelago::handle_command(&ctx, &msg).await {
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
