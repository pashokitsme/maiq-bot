use reqwest::Url;
use std::ops::Deref;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode, User},
  Bot,
};

use super::Command;
use crate::{bot::TeloxideResult, db::Mongo, error::TeloxideError};

// todo: make it injectable
/// M`essage` handler context
pub struct MContext {
  bot: Bot,
  pub msg: Message,
  pub user: User,
  pub used_command: Command,
  pub mongo: Mongo,
}

impl Deref for MContext {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl MContext {
  pub fn new(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> Self {
    Self { bot, user: msg.from().unwrap().clone(), msg, used_command: cmd, mongo }
  }

  pub fn chat_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<Message, TeloxideError> {
    Ok(self.bot.send_message(self.chat_id(), text).await?)
  }

  pub async fn send_about(&self) -> TeloxideResult {
    macro_rules! buttons_column {
      ($(($name: literal, $url: literal)),*) => {
        vec![$(vec![InlineKeyboardButton::url($name, Url::parse($url).unwrap()); 1]),*]
      };
    }

    let markup = InlineKeyboardMarkup::new(buttons_column!(("GitHub", "https://github.com/pashokitsme")));
    let msg = format!("<b>Информация</b>\n\nЗаглушка :c");
    self
      .send_message(self.chat_id(), msg)
      .parse_mode(ParseMode::Html)
      .reply_markup(markup)
      .await?;
    Ok(())
  }

  pub async fn toggle_notifications(&self) -> TeloxideResult {
    Ok(())
  }

  pub async fn set_group(&self, group: &String) -> TeloxideResult {
    Ok(())
  }

  pub async fn update(&self) -> TeloxideResult {
    Ok(())
  }
}
