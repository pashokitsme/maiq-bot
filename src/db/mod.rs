use mongodb::{options::ClientOptions, Collection};

use crate::env;

use self::models::User;

mod models;

pub type Mongo = mongodb::Client;
pub type MongoError = mongodb::error::Error;

pub async fn init() -> Result<Mongo, MongoError> {
  let url = env::var(env::DB_URL).unwrap();
  info!("Connection to database");
  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq-bot".into());
  opts.default_database = Some(env::var(env::DEFAULT_DB).unwrap());
  Mongo::with_options(opts)
}

fn get_users(db: &Mongo) -> Collection<User> {
  db.default_database().unwrap().collection("users")
}
