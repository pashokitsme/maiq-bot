pub mod handler;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::{
  payloads::AnswerCallbackQuerySetters,
  requests::Requester,
  types::{CallbackQuery, InlineKeyboardButton},
  Bot,
};

use crate::{
  bot::callbacks::handler::{delete_message, ok},
  db::MongoPool,
};

use super::{BotResult, Dispatch};

#[derive(Debug)]
pub struct Callback<T: Into<String>> {
  pub text: T,
  pub kind: CallbackKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackKind {
  Ok,
  Del,
  Unknown,
}

impl<T: Into<String>> Callback<T> {
  #[allow(dead_code)]
  pub fn new(text: T, kind: CallbackKind) -> InlineKeyboardButton {
    let data = String::from_utf8(bincode::serialize(&kind).unwrap()).unwrap();
    debug!("{:?} serialized to {:?}", kind, data);
    InlineKeyboardButton::callback(text, data)
  }
}

#[async_trait]
impl Dispatch for CallbackKind {
  type Kind = CallbackQuery;

  async fn dispatch(self, bot: Bot, q: Self::Kind, _mongo: MongoPool) -> BotResult {
    type K = CallbackKind;
    match self {
      K::Ok => ok(bot, q).await,
      K::Del => delete_message(bot, q).await,
      K::Unknown => {
        bot
          .answer_callback_query(q.id)
          .text("Я не знаю, что делать с этой кнопкой 🤕")
          .show_alert(true)
          .await?;
        Ok(())
      }
    }
  }
}
