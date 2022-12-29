use teloxide::{
  requests::Requester,
  types::{ChatId, Message, User},
  Bot,
};

use crate::error::TeloxideError;

use super::Command;

pub type Ok = Result<(), TeloxideError>;

pub struct HContext<'c> {
  pub bot: &'c Bot,
  pub msg: &'c Message,
  pub user: &'c User,
  pub used_command: Command,
}

impl<'c> HContext<'c> {
  pub fn new(bot: &'c Bot, msg: &'c Message, cmd: Command) -> Self {
    Self { bot, msg, user: msg.from().unwrap(), used_command: cmd }
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
