#[cfg(feature = "dev")] 
extern crate dotenv;

#[cfg(feature = "dev")]
use {
  dotenv::dotenv
};

use rand::Rng;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use std::collections::HashMap;
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
#[commands(ami, roll, r)]
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
async fn r(ctx: &Context, msg: &Message) -> CommandResult {
  match msg.content.roll() {
    Ok(n) => msg.reply(ctx, format!("```{}```", n.to_string())).await?,
    Err(e) => msg.reply(ctx, format!("{}", e)).await?
  };

  Ok(())
}

#[command]
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
  match msg.content.roll() {
    Ok(n) => msg.reply(ctx, format!("```{}```", n.to_string())).await?,
    Err(e) => msg.reply(ctx, format!("{}", e)).await?
  };

  Ok(())
}

#[command]
async fn ami(ctx: &Context, msg: &Message) -> CommandResult {
  let wisdom = format!(
    "`Yes, {}, you are {}.`"
    , msg.author.name
    , msg.content
      .strip_command()
      .replace("?", "")
  ).to_string();

  msg.reply(ctx, wisdom).await?;

  Ok(())
}


pub trait Contentful {
  fn strip_command(&self) -> String;
}


impl Contentful for str {
  fn strip_command(&self) -> String{
    let words: Vec<&str> = self.split(" ").collect();
    words[1..].join(" ")
  }
}


impl ToString for RollResult {
  fn to_string(&self) -> String {
    let mut output = if self.rolls.len() == 1 { String::new() } else { format!("Result: {}.", self.total) };
    let rolls = &self.rolls;

    for (_key, value) in rolls.into_iter() {
      output = format!("{}\n {} x d{} - {}", output, value.multiplier, value.faces, value.total);

      if value.results.len() > 1  && value.results.len() < 10 {
        for r in &value.results {
          output = format!("{}\n  {}", output, r);
        }
      } else if value.results.len() > 10 {
        output = format!("{}\n  >: | Thats a lot of dice, you'll just have to trust me.", output);
      }
    }
    output
  }
}


pub struct RollResult {
  rolls: HashMap<String, RollSet>,
  total: u128
}


pub struct RollSet {
  faces: u16,
  multiplier: u128,
  total: u128,
  results: Vec<u128>
}


pub trait Rollable {
  fn roll(&self) -> Result<RollResult, String>;
}


impl Rollable for str {
  fn roll(&self) -> Result<RollResult, String> {

    let mut roll_result = RollResult {
      rolls: HashMap::new(),
      total: 0
    };

    println!("parsing {}", self);

    let mut clean_string: String = self
      .strip_command()
      .trim()
      .chars()
      .filter(|c| c.is_numeric() || c == &' ' || c.to_lowercase().to_string() == "d")
      .collect();

    while clean_string.contains("  ") {
      clean_string = clean_string.replace("  ", " ")
    }

    let mut is_seeking_multiplier = true;
    let mut current_multiplier =  if clean_string.chars().next().unwrap() == 'd' { "1".to_string() } else { String::new() };
    let mut chars = clean_string.chars().peekable();
    let mut current_faces = String::new();

    println!("Looking for multipliers:");
    loop {
      match chars.next() {
        Some(c) => {
          println!("  current letter: {}", c);
          if is_seeking_multiplier {
            match c.is_numeric() || c == ' ' {
              true => if c != ' ' { 
                current_multiplier.push(c); 
                println!("found multiplier value");
              },
              false => {
                if c == 'd' && chars.peek().unwrap() == &' ' {
                  chars.next();
                }
                is_seeking_multiplier = false;
                println!("Looking for faces.");
              }
            };
          } else {
            match c.is_numeric() && chars.peek().is_some() {
              true => current_faces.push(c),
              false => {
                if chars.peek().is_none() {
                  if c.is_numeric() { current_faces.push(c) }
                } else {
                  println!("|{}|", c.to_string());
                  if c == 'd' && chars.peek().unwrap() == &' ' {
                    continue
                  }
                }
                match (current_multiplier.parse::<u16>(), current_faces.parse::<u16>()) {
                  (Ok(multiplier), Ok(faces)) => {
                    println!("Nice! adding dice roll: {} x d{}", multiplier, faces);
                    let rolls = roll_result.rolls
                      .entry(format!("d{}", faces))
                      .or_insert(RollSet { 
                        faces: faces
                        , multiplier: 0
                        , total: 0
                        , results: Vec::new()
                    });
                    
                    for _i in 0..multiplier {
                      let mut rng = rand::thread_rng();
                      let roll = rng.gen_range(1..rolls.faces) as u128;
                      rolls.results.push(roll);
                      rolls.total += roll as u128;
                      roll_result.total += roll as u128;
                    }
                    rolls.multiplier += multiplier as u128;

                    if c == 'd' {
                      current_multiplier = "1".to_string();
                    } else {
                      current_multiplier = String::new();
                      is_seeking_multiplier = true;
                    }
                    current_faces = String::new();
                    println!("Looking for multipliers.");
                  },
                  (Err(e), Err(e2)) => return Err(format!(
                    "```\nNeither '{}' nor '{}' make a parsable dice roll:\n {} \n\n  ERROR: {} \n VALID DICE: {}```"
                    , current_multiplier
                    , current_faces
                    , e
                    , e2
                    , roll_result.to_string()
                  )),
                  (Ok(_n), Err(e)) => return Err(format!(
                    "```\nThe multiplier '{}' looks right to me, but d'{}' cannot be parsed as a dice string:\n\n  ERROR: {}, \n VALID DICE: {}```"
                    , current_multiplier
                    , current_faces
                    , e
                    , roll_result.to_string()
                  )),
                  (Err(e), Ok(_n)) => return Err(format!(
                    "\n```'{}' cannot be parsed as a multiplier, but  d'{}' looks like a nice dice string:\n\n  ERROR: {}```"
                    , current_multiplier
                    , current_faces,
                     e
                  ))
                };
              }
            }
          }
        }
        _ => break,
      }
    }
    Ok(roll_result)
  }
}


#[test]
fn test_roll() {
  match "!roll 1d202d40".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
  match "!roll 1d2ferfag23498d892838  j j j j34d 3 j34j d0fefa2d40".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
  match "!roll 1d2ferfag0fefa2d40".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
  match "!roll 1d 480598309 fwaefj efjij 4d5t969 fejfeijijfj4d438".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
  match "!roll d20".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
}
