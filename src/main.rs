use teloxide::Bot;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

mod api;
mod bot;
mod db;
mod env;
mod error;

#[tokio::main]
async fn main() {
  init();
  let mongo = db::init().await.expect("Couldn't connect to database");
  let bot = Bot::from_env();
  bot::start(bot, mongo).await
}

fn init() {
  dotenvy::dotenv().expect("Unable to init .env");
  pretty_env_logger::init();
  env::check_env_vars();
}
