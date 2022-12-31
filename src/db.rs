use crate::models::User;
use mongodb::{bson::doc, options::ClientOptions, Collection};
use teloxide::types::UserId;

use crate::env;

pub type Mongo = mongodb::Client;
pub type MongoError = mongodb::error::Error;

pub async fn init() -> Result<Mongo, MongoError> {
  let url = env::var(env::DB_URL).unwrap();
  info!("Connecting to database");
  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq-bot".into());
  opts.default_database = Some(env::var(env::DEFAULT_DB).unwrap());
  Mongo::with_options(opts)
}

pub async fn get_user(db: &Mongo, id: i64) -> Result<Option<User>, MongoError> {
  Ok(get_users(&db).find_one(doc! { "id": id }, None).await?)
}

pub async fn update_user(db: &Mongo, user: &User) -> Result<Option<User>, MongoError> {
  let users = get_users(&db);
  Ok(users.find_one_and_replace(doc! { "id": user.id}, user, None).await?)
}

pub async fn new_user(db: &Mongo, id: i64) -> Result<(), MongoError> {
  let users = get_users(&db);
  if get_user(&db, id).await?.is_some() {
    warn!("Tryed to insert new user but user with id {} already exists", id);
    return Ok(());
  }
  let user = User::new(UserId(id as u64));
  users.insert_one(user, None).await?;
  Ok(())
}

fn get_users(db: &Mongo) -> Collection<User> {
  db.default_database().unwrap().collection("users")
}
