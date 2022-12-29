use teloxide::Bot;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

mod api;
mod bot;
mod env;
mod error;

#[tokio::main]
async fn main() {
  init();
  let bot = Bot::from_env();
  bot::start(bot).await
}

fn init() {
  dotenvy::dotenv().expect("Unable to init .env");
  pretty_env_logger::init();
  env::check_env_vars();
}
