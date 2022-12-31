use maiq_structs::{Group, Snapshot};
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
  api,
  bot::TeloxideResult,
  db::{self, Mongo},
  error::TeloxideError,
  utils,
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

  #[allow(deprecated)]
  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<Message, TeloxideError> {
    Ok(
      self
        .bot
        .send_message(self.chat_id(), text)
        .parse_mode(ParseMode::Markdown)
        .await?,
    )
  }

  pub async fn send_timetable(snapshot: &Snapshot) -> TeloxideResult {
    unimplemented!()
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
    let msg = format!("<b>Информация</b>\n\nЗаглушка :(");
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
        .reply(format!("Ты уверен, что именно <code>{}</code>? По моему, слишком длинное название.", group))
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

  // todo: await db & api requests in same time
  pub async fn update(&self) -> TeloxideResult {
    let user = db::get_or_create_user(&self.mongo, self.user.id.0 as i64).await?;
    if user.group.is_none() {
      self.reply("Ты не установил группу").await?;
      return Ok(());
    }
    let group = user.group.unwrap();

    let today = api::get_latest_today().await;
    let next = api::get_latest_next().await;
    if today.is_err() && next.is_err() {
      self.reply("У меня нет расписания ни на сегодня, ни на завтра. Может, стоит посмотреть на [сайте](http://chemk.org/index.php/raspisanie>)?").await?;
      return Ok(());
    }
    let mut message = String::new();

    if let Ok(today) = today {
      if let Some(today) = today.groups.iter().find(|g| g.name == group) {
        message.push_str("<b>Расписание на сегодня</b>\n");
        today
          .lessons
          .iter()
          .for_each(|l| message.push_str(utils::display_lesson(&l).as_str()));
        message.push('\n');
      }
    }

    if let Ok(next) = next {
      if let Some(next) = next.groups.iter().find(|g| g.name == group) {
        message.push_str("<b>Расписание на следующий день</b>\n");
        next
          .lessons
          .iter()
          .for_each(|l| message.push_str(utils::display_lesson(&l).as_str()))
      }
    }

    self.reply(message).await?;
    Ok(())
  }
}
