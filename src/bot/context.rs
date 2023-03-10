use std::ops::Deref;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
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

  pub async fn toggle_notifications(&self) -> BotResult {
    let mut user = self.mongo.get_or_new(self.user_id()).await?;
    user.is_notifications_enabled = !user.is_notifications_enabled;
    self.mongo.update(&user).await?;
    self.reply(format!("{}", user.is_notifications_enabled)).await?;
    Ok(())
  }

  pub async fn set_group(&self, group: &String) -> BotResult {
    if group.is_empty() || group.len() > 10 {
      return Err(BotError::invalid_command("/set_group", "/set_group [группа: длина &lt; 10]", "/set_group Ир3-21"));
    }

    let mut user = self.mongo.get_or_new(self.user_id()).await?;
    user.group = Some(group.clone());
    user.is_notifications_enabled = true;
    self.mongo.update(&user).await?;
    self
      .reply(format!("Теперь твоя группа: <code>{}</code>", user.group.unwrap()))
      .await?;
    Ok(())
  }
}
