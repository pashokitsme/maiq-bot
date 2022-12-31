use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use teloxide::types::UserId;

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
