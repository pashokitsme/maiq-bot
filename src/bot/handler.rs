use reqwest::Url;
use std::ops::Deref;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode, User},
  Bot,
};

use super::Command;
use crate::{
  bot::TeloxideResult,
  db::{self, Mongo},
  error::TeloxideError,
};

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

  pub async fn start_command(&self) -> TeloxideResult {
    _ = db::get_or_create_user(&self.mongo, self.user.id.0 as i64).await?;
    self
      .reply("Привет. Это бета.\nТебе нужно установить свою группу при помощи команды /set_group [группа]")
      .await?;
    Ok(())
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
    let mut user = db::get_or_create_user(&self.mongo, self.user.id.0 as i64).await?;
    user.is_notifications_enabled = !user.is_notifications_enabled;
    db::update_user(&self.mongo, &user).await?;
    self.reply(format!("{}", user.is_notifications_enabled)).await?;
    Ok(())
  }

  pub async fn set_group(&self, group: &String) -> TeloxideResult {
    if group.is_empty() {
      self
        .reply("Ты должен передать название группы в аргумент команды")
        .await?;
      return Ok(());
    }

    if group.len() > 6 {
      self
        .reply(format!("Ты уверен, что именно `{}`? По моему, слишком длинное название.", group))
        .await?;
      return Ok(());
    }

    let mut user = db::get_or_create_user(&self.mongo, self.user.id.0 as i64).await?;
    user.group = Some(group.clone());
    db::update_user(&self.mongo, &user).await?;
    if user.group.is_none() {
      self.toggle_notifications().await?;
    }
    self
      .reply(format!("Теперь твоя группа: {}", user.group.unwrap()))
      .await?;
    Ok(())
  }

  pub async fn update(&self) -> TeloxideResult {
    Ok(())
  }
}
