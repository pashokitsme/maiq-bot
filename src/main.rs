use poller::Poller;
use teloxide::Bot;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

mod bot;
mod db;
mod env;
mod error;
mod poller;

#[tokio::main]
async fn main() {
  dotenvy::dotenv().ok();
  pretty_env_logger::init();
  env::check_env_vars();

  let mongo = db::MongoPool::init().await.expect("Couldn't connect to database");
  let bot = Bot::from_env();

  let mut poller = Poller::new(bot.clone(), mongo.clone());
  tokio::spawn(async move { poller.run().await });

  bot::start(bot, mongo).await
}
