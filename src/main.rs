use dotenv::dotenv;
use serenity::{async_trait, Client};
use serenity::all::{Context, EventHandler, GatewayIntents, Ready};

mod rarety;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.tag())
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let token = std::env::var("TOKEN").expect("Failed to find token");
    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Error creating client");

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}");
    }
}
