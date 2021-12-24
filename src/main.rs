#[cfg(feature = "dev")] 
extern crate dotenv;

#[cfg(feature = "dev")]
use {
  dotenv::dotenv
};

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::framework::standard::{
  StandardFramework,
  CommandResult,
  macros::{
    command,
    group
  }
};

use std::env;

#[group]
#[commands(ping,ami)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
  #[cfg(feature = "dev")]
  dotenv().ok();

  let framework = StandardFramework::new()
    .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
    .group(&GENERAL_GROUP);

  // Login with a bot token from the environment
  let token = env::var("DISCORD_TOKEN").expect("token");
  let mut client = Client::builder(token)
    .event_handler(Handler)
    .framework(framework)
    .await
    .expect("Error creating client");

  // start listening for events by starting a single shard
  if let Err(why) = client.start().await {
    println!("An error occurred while running the client: {:?}", why);
  }
}


#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  msg.reply(ctx, "Pong!").await?;

  Ok(())
}


#[command]
async fn ami(ctx: &Context, msg: &Message) -> CommandResult {
  let words: Vec<&str> = msg.content.split(" ").collect();
  let strip_command_and_rephrase = words[1..].join(" ").replace("?", ".");
  msg.reply(ctx, format!("`Yes, {}, you are {}.`", msg.author.name, strip_command_and_rephrase)).await?;

  Ok(())
}
