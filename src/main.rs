#[cfg(feature = "dev")] 
extern crate dotenv;

#[cfg(feature = "dev")]
use {
  dotenv::dotenv
};

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
#[commands(ami, roll)]
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
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
  match msg.content.roll() {
    Ok(n) => msg.reply(ctx, format!("hi! {}", n.total)).await?,
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


pub trait Faceted {
  fn face_count(&self) -> Result<u16, String>;
}


impl Faceted for str {
  fn face_count(&self) -> Result<u16, String> {
    let number_chars: Vec<char> = self.chars().filter(|c| c.is_numeric()).collect();
    let number_string: String = number_chars.iter().collect();

    match number_string.parse::<u16>() {
      Ok(n) => Ok(n),
      Err(e) => Err(format!("That number sucks: {}", e)),
    }
  }
}


pub struct RollResult {
  rolls: HashMap<String, RollSet>,
  total: u128
}

impl ToString for RollResult {
  fn to_string(&self) -> String {
    let mut output = if self.rolls.len() == 1 { String::new() } else { format!("Result: {}.", self.total) };
    let rolls = &self.rolls;

    for (_key, value) in rolls.into_iter() {
      output = format!("{}\n {} x d{} - {}", output, value.multiplier, value.faces, value.total);

      if value.results.len() > 1 {
        for r in &value.results {
          output = format!("{}\n  {}", output, r);
        }
      }
    }
    output
  }
}

pub struct RollSet {
  faces: u16,
  multiplier: u16,
  total: u128,
  results: Vec<u32>
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

    println!("first pass: |{}|", clean_string);

    while clean_string.contains("  ") {
      clean_string = clean_string.replace("  ", " ")
    }

    println!("second pass: |{}|", clean_string);

    let mut is_seeking_multiplier = true;
    let mut current_multiplier = String::new();
    let mut current_faces = String::new();
    let chars = clean_string.chars().peekable();

    println!("Looking for multipliers:");
    while let c = chars.next() {
      println!("  current letter: {}", c.unwrap());

      if is_seeking_multiplier {
        match c.unwrap().is_numeric() || c.unwrap() == ' ' {
          true => if c.unwrap() != ' ' { 
            current_multiplier.push(c.unwrap()); 
            println!("found multiplier value");
          },
          false => {
            is_seeking_multiplier = false;
            println!("Looking for faces.");
          }
        };

      } else {

        match c.unwrap().is_numeric() && chars.peek().is_some() {
          true => current_faces.push(c.unwrap()),
          false => {
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

                rolls.multiplier += multiplier;
                let result = (multiplier * faces) as u32;
                roll_result.total += result as u128;
                rolls.total += result as u128;
                rolls.results.push(result);
                current_faces = String::new();
                current_multiplier = String::new();
                is_seeking_multiplier = true;
                println!("Looking for multipliers.");
              },
              (Err(e), Err(e2)) => return Err(format!("\nNeither '{}' nor '{}' make a parsable dice roll:\n {} \n\n  ERROR: {}", current_multiplier, current_faces, e, e2)),
              (Ok(_n), Err(e)) => return Err(format!("\nThe multiplier '{}' looks right to me, but d'{}' cannot be parsed as a dice string:\n\n  ERROR: {}", current_multiplier, current_faces, e)),
              (Err(e), Ok(_n)) => return Err(format!("\n'{}' cannot be parsed as a multiplier, but  d'{}' looks like a nice dice string:\n\n  ERROR: {}", current_multiplier, current_faces, e))
            };
          }
        }
      }
    }
    match (current_multiplier.parse::<u16>(), current_faces.parse::<u16>()) {
      (Ok(multiplier), Ok(faces)) => {
        println!("Nice! adding dice roll: {} x d{}", multiplier, faces);
        let rolls = roll_result.rolls
          .entry(format!("d{}", faces))
          .or_insert(RollSet { 
            faces: 0
            , multiplier: 0
            , total: 0
            , results: Vec::new()
        });

        rolls.multiplier += multiplier;
        rolls.faces = faces;
        let result = (multiplier * faces) as u32;
        roll_result.total += result as u128;
        rolls.total += result as u128;
        rolls.results.push(result);
        println!("Looking for multipliers.");

      },
      (Err(e), Err(e2)) => return Err(format!("\nNeither '{}' nor '{}' make a parsable dice roll:\n {} \n\n  ERROR: {}", current_multiplier, current_faces, e, e2)),
      (Ok(_n), Err(e)) => return Err(format!("\nThe multiplier '{}' looks right to me, but d'{}' cannot be parsed as a dice string:\n\n  ERROR: {}", current_multiplier, current_faces, e)),
      (Err(e), Ok(_n)) => return Err(format!("\n'{}' cannot be parsed as a multiplier, but  d'{}' looks like a nice dice string:\n\n  ERROR: {}", current_multiplier, current_faces, e))
    }
    Ok(roll_result)
  }
}


#[test]
fn test_dice_conversion() {
  assert_eq!("d20".face_count(), Ok(20));
  assert_eq!("d20fj999bblkjh".face_count(), Ok(20999));
  assert!("d20fj999bbl9999kjh999a99999999".face_count().is_err());
}

#[test]
fn test_roll() {
  match "!roll 1d202d40".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
  match "!roll 1d2ferfag0fefa2d40".roll() {
    Ok(n) => println!("{}", n.to_string()),
    Err(e) => println!("{}", e) 
  };
}
