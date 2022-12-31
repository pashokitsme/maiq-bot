use mongodb::{
  bson::{doc, DateTime},
  options::ClientOptions,
  Collection,
};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub id: i64,
  pub group: Option<String>,
  pub is_notifications_enabled: bool,
  pub joined: DateTime,
}

impl User {
  pub fn new(id: UserId) -> Self {
    Self { id: id.0 as i64, is_notifications_enabled: false, joined: DateTime::now(), group: None }
  }
}

pub async fn get_user(db: &Mongo, id: i64) -> Result<Option<User>, MongoError> {
  Ok(get_users(&db).find_one(doc! { "id": id }, None).await?)
}

pub async fn get_or_create_user(db: &Mongo, id: i64) -> Result<User, MongoError> {
  let users = get_users(&db);
  match users.find_one(doc! { "id": id }, None).await? {
    Some(user) => return Ok(user),
    None => return Ok(new_user(&db, id).await?),
  }
}

pub async fn update_user(db: &Mongo, user: &User) -> Result<Option<User>, MongoError> {
  Ok(
    get_users(&db)
      .find_one_and_replace(doc! { "id": user.id}, user, None)
      .await?,
  )
}

pub async fn new_user(db: &Mongo, id: i64) -> Result<User, MongoError> {
  let users = get_users(&db);
  if let Some(user) = get_user(&db, id).await? {
    warn!("Tryed to insert new user but user with id {} already exists", id);
    return Ok(user);
  }
  let user = User::new(UserId(id as u64));
  info!("New user with id {}", user.id);
  users.insert_one(&user, None).await?;
  Ok(user)
}

fn get_users(db: &Mongo) -> Collection<User> {
  db.default_database().unwrap().collection("users")
}
