#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

mod api;
mod env;

#[tokio::main]
async fn main() {
  init();
}

fn init() {
  dotenvy::dotenv().expect("Unable to init .env");
  pretty_env_logger::init();
  env::check_env_vars();
}
