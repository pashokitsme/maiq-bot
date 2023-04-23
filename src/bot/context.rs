use std::ops::Deref;
use teloxide::{
  payloads::{SendMessage, SendMessageSetters},
  requests::{JsonRequest, Requester},
  types::{ChatId, Message, ParseMode, UserId},
  Bot,
};

use crate::{bot::BotResult, db::MongoPool, error::BotError};

pub struct Context {
  bot: Bot,
  pub msg: Message,
  pub mongo: MongoPool,
}

impl Deref for Context {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl Context {
  pub fn new(bot: Bot, msg: Message, mongo: MongoPool) -> Self {
    Self { bot, msg, mongo }
  }

  pub fn chat_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub fn user_id(&self) -> UserId {
    self.msg.from().map(|f| f.id).unwrap_or(UserId(0))
  }

  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<(), BotError> {
    self
      .bot
      .send_message(self.chat_id(), text)
      .parse_mode(ParseMode::Html)
      .disable_web_page_preview(true)
      .await?;
    Ok(())
  }

  pub fn reply_ex<T: Into<String>>(&self, text: T) -> JsonRequest<SendMessage> {
    self
      .bot
      .send_message(self.chat_id(), text)
      .parse_mode(ParseMode::Html)
      .disable_web_page_preview(true)
  }

  pub async fn toggle_notifications(&self) -> BotResult {
    let mut user = self.mongo.get_or_new(self.user_id()).await?;
    user.is_notifications_enabled = !user.is_notifications_enabled;
    self.mongo.update(&user).await?;
    self.reply(format!("{}", user.is_notifications_enabled)).await?;
    Ok(())
  }

  pub async fn set_teacher(&self, name: &str) -> BotResult {
    let mut user = self.mongo.get_or_new(self.user_id()).await?;
    match name {
      "" => {
        user.teacher = None;
        self.reply("Имя удалено").await?
      }
      x => {
        user.teacher = Some(x.into());
        self.reply(format!("Имя: {}", name)).await?;
      }
    };
    self.mongo.update(&user).await?;
    Ok(())
  }
}
