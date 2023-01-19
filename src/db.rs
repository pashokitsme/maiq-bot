use std::ops::Deref;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
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

impl Settings {
  pub fn new(id: UserId) -> Self {
    Self { id: id.0 as i64, is_notifications_enabled: false, joined: DateTime::now(), group: None }
  }
}

#[derive(Clone)]
pub struct MongoPool {
  mongo: Mongo,
  settings: Collection<Settings>,
}

impl Deref for MongoPool {
  type Target = Mongo;

  fn deref(&self) -> &Self::Target {
    &self.mongo
  }
}

impl MongoPool {
  pub async fn init() -> Result<Self, MongoError> {
    let url = env::var(env::DB_URL).unwrap();
    info!("Connecting to database");
    let mut opts = ClientOptions::parse(url).await?;
    opts.app_name = Some("maiq-bot".into());
    opts.default_database = Some(env::var(env::DEFAULT_DB).unwrap());
    let mongo = Mongo::with_options(opts)?;
    let settings = mongo.default_database().unwrap().collection("users");
    Ok(Self { mongo, settings })
  }

  pub async fn get(&self, id: i64) -> Result<Option<Settings>, MongoError> {
    Ok(self.settings.find_one(doc! { "id": id }, None).await?)
  }

  pub async fn get_or_new(&self, id: i64) -> Result<Settings, MongoError> {
    match self.settings.find_one(doc! { "id": id }, None).await? {
      Some(user) => Ok(user),
      None => Ok(self.new(id).await?),
    }
  }

  pub async fn update(&self, new_settings: &Settings) -> Result<Option<Settings>, MongoError> {
    Ok(
      self
        .settings
        .find_one_and_replace(doc! { "id": new_settings.id }, new_settings, None)
        .await?,
    )
  }

  pub async fn new(&self, id: i64) -> Result<Settings, MongoError> {
    if let Some(user) = self.get(id).await? {
      warn!("Tried to insert new user but user with id {} already exists", id);
      return Ok(user);
    }
    let user = Settings::new(UserId(id as u64));
    info!("New user with id {}", user.id);
    self.settings.insert_one(&user, None).await?;
    Ok(user)
  }

  pub async fn notifiables<'a>(&self) -> Result<Vec<Notifiable>, BotError> {
    info!("Colleting notifiable users");
    let mut notifies: Vec<Notifiable> = vec![];
    let mut cur = self
      .settings
      .find(doc! { "is_notifications_enabled": true }, None)
      .await?;
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
}
