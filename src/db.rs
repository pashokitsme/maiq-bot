use mongodb::{
  bson::{doc, DateTime},
  options::ClientOptions,
  Collection,
};
use serde::{Deserialize, Serialize};
use teloxide::types::UserId;

use crate::{env, error::BotError};

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
pub struct UserSettings {
  pub id: i64,
  pub group: Option<String>,
  pub is_notifications_enabled: bool,
  pub joined: DateTime,
}

#[derive(Debug)]
pub struct Notifiable {
  pub group: String,
  pub user_ids: Vec<i64>,
}

impl Notifiable {
  pub fn new(group: String, id: i64) -> Self {
    Notifiable { group, user_ids: vec![id] }
  }
}

impl UserSettings {
  pub fn new(id: UserId) -> Self {
    Self { id: id.0 as i64, is_notifications_enabled: false, joined: DateTime::now(), group: None }
  }
}

pub async fn get_user_settings(db: &Mongo, id: i64) -> Result<Option<UserSettings>, MongoError> {
  Ok(user_settings_models(&db).find_one(doc! { "id": id }, None).await?)
}

pub async fn get_or_create_user_settings(db: &Mongo, id: i64) -> Result<UserSettings, MongoError> {
  let users = user_settings_models(&db);
  match users.find_one(doc! { "id": id }, None).await? {
    Some(user) => Ok(user),
    None => Ok(new_user_settings(&db, id).await?),
  }
}

pub async fn update_user_settings(db: &Mongo, user: &UserSettings) -> Result<Option<UserSettings>, MongoError> {
  Ok(
    user_settings_models(&db)
      .find_one_and_replace(doc! { "id": user.id }, user, None)
      .await?,
  )
}

pub async fn new_user_settings(db: &Mongo, id: i64) -> Result<UserSettings, MongoError> {
  let users = user_settings_models(&db);
  if let Some(user) = get_user_settings(&db, id).await? {
    warn!("Tried to insert new user but user with id {} already exists", id);
    return Ok(user);
  }
  let user = UserSettings::new(UserId(id as u64));
  info!("New user with id {}", user.id);
  users.insert_one(&user, None).await?;
  Ok(user)
}

pub async fn get_notifiables<'a>(db: &'a Mongo) -> Result<Vec<Notifiable>, BotError> {
  info!("Colleting notifiable users");
  let users = user_settings_models(&db);
  let mut notifies: Vec<Notifiable> = vec![];
  let mut cur = users.find(doc! { "is_notifications_enabled": true }, None).await?;
  while cur.advance().await? {
    let raw = cur.current();
    let id = raw.get_i64("id");
    let group = raw.get_str("group");
    if id.is_err() || group.is_err() {
      continue;
    }

    let id = id.unwrap().clone();
    let group = group.unwrap().clone();

    match notifies.iter_mut().find(|n| n.group == group) {
      Some(n) => n.user_ids.push(id),
      None => notifies.push(Notifiable::new(group.to_string(), id)),
    }
  }

  Ok(notifies)
}

fn user_settings_models(db: &Mongo) -> Collection<UserSettings> {
  db.default_database().unwrap().collection("users")
}
