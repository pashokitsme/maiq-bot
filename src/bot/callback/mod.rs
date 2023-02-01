pub mod handler;

use serde::{Deserialize, Serialize};
use teloxide::{
  types::{CallbackQuery, InlineKeyboardButton},
  Bot,
};

use crate::bot::callback::handler::{delete_message, ok};

use super::BotResult;

#[derive(Debug)]
pub struct Callback<T: Into<String>> {
  pub text: T,
  pub kind: CallbackKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackKind {
  Ok,
  Del,
}

impl<T: Into<String>> Callback<T> {
  pub fn new(text: T, kind: CallbackKind) -> InlineKeyboardButton {
    let data = String::from_utf8(bincode::serialize(&kind).unwrap()).unwrap();
    debug!("{:?} serialized to {:?}", kind, data);
    InlineKeyboardButton::callback(text, data)
  }
}

pub async fn handle(bot: Bot, q: CallbackQuery, kind: CallbackKind) -> BotResult {
  type K = CallbackKind;
  match kind {
    K::Ok => ok(bot, q).await,
    K::Del => delete_message(bot, q).await,
  }
}
