use std::ops::Deref;

use maiq_shared::utils::time::now;
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
  pub teacher: Option<String>,
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
    Self { id: id.0 as i64, is_notifications_enabled: false, joined: DateTime::from_chrono(now()), group: None, teacher: None }
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

  pub async fn get_or_new(&self, id: UserId) -> Result<Settings, MongoError> {
    if let Some(settings) = self.settings.find_one(doc! { "id": id.0 as i64 }, None).await? {
      return Ok(settings);
    }

    let user = Settings::new(id);
    self.settings.insert_one(&user, None).await?;
    info!("New user-id {}", user.id);
    Ok(user)
  }

  pub async fn update(&self, new_settings: &Settings) -> Result<Option<Settings>, MongoError> {
    self
      .settings
      .find_one_and_replace(doc! { "id": new_settings.id }, new_settings, None)
      .await
  }

  #[allow(dead_code)] // todo: disallow
  pub async fn delete(&self, id: UserId) -> Result<(), MongoError> {
    self.settings.delete_one(doc! { "id": id.0 as i64}, None).await?;
    info!("Deleted user-id {}", id.0);
    Ok(())
  }

  pub async fn notifiables(&self) -> Result<Vec<Notifiable>, BotError> {
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

      let id = id.unwrap();
      let group = group.unwrap();

      match notifies.iter_mut().find(|n| n.group == group) {
        Some(n) => n.user_ids.push(id),
        None => notifies.push(Notifiable::new(group.to_string(), id)),
      }
    }

    Ok(notifies)
  }

  pub async fn fetch_all(&self) -> Result<Vec<Settings>, BotError> {
    let mut result = vec![];
    let mut cur = self.settings.find(doc! {}, None).await?;
    while cur.advance().await? {
      result.push(cur.deserialize_current()?);
    }

    Ok(result)
  }

  pub async fn fetch_all_notifiable_ids(&self) -> Result<Vec<i64>, BotError> {
    let mut result = vec![];
    let mut cur = self
      .settings
      .find(doc! { "is_notifications_enabled": true }, None)
      .await?;

    while cur.advance().await? {
      let id = match cur.current().get_i64("id") {
        Ok(id) => id,
        Err(_) => continue,
      };

      result.push(id);
    }
    Ok(result)
  }
}
