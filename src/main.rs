use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let token =
        std::env::var("GOBY_TOKEN").expect("Missing GOBY_TOKEN environment variable");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions::<Data, Error> {
            commands: vec![],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
