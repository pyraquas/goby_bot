use poise::serenity_prelude as serenity;

mod admin;
mod mtg;

/// State shared by every command invocation.
pub struct Data {
    /// Card lists registered with `/have`.
    pub mtg: mtg::store::Store,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Loads the project root .env, real environment variables win over it.
    dotenvy::dotenv().expect("Failed to read .env file");
    let token = std::env::var("GOBY_TOKEN").expect("Missing GOBY_TOKEN environment variable");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions::<Data, Error> {
            commands: vec![
                admin::commands::init(),
                mtg::commands::have(),
                mtg::commands::need(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    mtg: mtg::store::Store::load().await?,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::InteractionCreate { interaction } = event
        && let Some(component) = interaction.as_message_component()
    {
        admin::role_select::handle(ctx, component).await?;
    }

    Ok(())
}
