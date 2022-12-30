use std::ops::Deref;
use teloxide::{
  requests::Requester,
  types::{ChatId, Message, User},
  Bot,
};

use crate::error::TeloxideError;

use super::Command;

pub type Ok = Result<(), TeloxideError>;

// todo: make it injectable
pub struct HContext {
  pub bot: Bot,
  pub msg: Message,
  pub user: User,
  pub used_command: Command,
}

impl Deref for HContext {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl HContext {
  pub fn new(bot: Bot, msg: Message, cmd: Command) -> Self {
    Self { bot, user: msg.from().unwrap().clone(), msg, used_command: cmd }
  }

  pub fn chat_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<Message, TeloxideError> {
    let sent = self.bot.send_message(self.chat_id(), text).await?;
    Ok(sent)
  }

  pub async fn say_hi(&self) -> Ok {
    self.reply("Здарова!").await?;
    Ok(())
  }
}
