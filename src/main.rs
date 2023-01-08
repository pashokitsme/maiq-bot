use background_poller::Poller;
use teloxide::Bot;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

mod api;
mod background_poller;
mod bot;
mod db;
mod env;
mod error;

#[tokio::main]
async fn main() {
  dotenvy::dotenv().expect("Unable to init .env");
  pretty_env_logger::init();
  env::check_env_vars();

  let mongo = db::init().await.expect("Couldn't connect to database");
  let bot = Bot::from_env();

  let bot_ref = bot.clone();
  let mut poller = Poller::new(bot_ref).await;
  tokio::spawn(async move { poller.run().await });

  let bot_ref = bot.clone();
  bot::start(bot_ref, mongo).await
}
