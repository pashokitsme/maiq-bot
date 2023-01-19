use chrono::{Datelike, NaiveDate};
use reqwest::Url;
use std::ops::Deref;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode, User},
  Bot,
};

use super::{format::DefaultFormatter, Command};
use crate::{api, bot::BotResult, db::MongoPool, error::BotError};

// todo: (?) make it injectable
/// M`essage` handler context
pub struct MContext {
  bot: Bot,
  pub msg: Message,
  pub user: User,
  pub used_command: Command,
  pub mongo: MongoPool,
}

impl Deref for MContext {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl MContext {
  pub fn new(bot: Bot, msg: Message, cmd: Command, mongo: MongoPool) -> Self {
    Self { bot, user: msg.from().unwrap().clone(), msg, used_command: cmd, mongo }
  }

  pub fn sender_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub fn sender_id_i64(&self) -> i64 {
    self.sender_id().0 as i64
  }

  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<(), BotError> {
    self
      .bot
      .send_message(self.sender_id(), text)
      .parse_mode(ParseMode::Html)
      .disable_web_page_preview(true)
      .await?;
    Ok(())
  }

  pub async fn start_n_init(&self) -> BotResult {
    _ = self.mongo.get_or_new(self.sender_id_i64()).await?;
    self
      .reply("Привет. Это что-то типо беты. По всем вопросам/багам/предложениям <a href=\"https://t.me/pashokitsme\">сюда</a>.\n\nКстати, в поиске хостинга.\nИ в ожидании звёздочек на <a href=\"https://github.com/pashokitsme\">гитхабе</a>! 🌟\n\nДля начала тебе нужно установить свою группу:\n<code>/set_group [группа]</code>\nПример:\n<code>/set_group Ир3-21</code>",)
      .await?;
    Ok(())
  }

  pub async fn reply_about(&self) -> BotResult {
    macro_rules! url_buttons_column {
      ($(($name: literal, $url: literal)),*) => {
        InlineKeyboardMarkup::new(vec![$(vec![InlineKeyboardButton::url($name, Url::parse($url).unwrap()); 1]),*])
      };
    }

    let markup = url_buttons_column!(
      ("По всем вопросам", "https://t.me/pashokitsme"),
      ("API", "https://github.com/pashokitsme/maiq-web-api"),
      ("GitHub", "https://github.com/pashokitsme")
    );
    let msg = format!("<b>Информация</b>\nЗаглушка :(");
    self
      .send_message(self.sender_id(), msg)
      .parse_mode(ParseMode::Html)
      .reply_markup(markup)
      .await?;
    Ok(())
  }

  pub async fn toggle_notifications(&self) -> BotResult {
    let mut user = self.mongo.get_or_new(self.sender_id_i64()).await?;
    user.is_notifications_enabled = !user.is_notifications_enabled;
    self.mongo.update(&user).await?;
    self.reply(format!("{}", user.is_notifications_enabled)).await?;
    Ok(())
  }

  pub async fn set_group(&self, group: &String) -> BotResult {
    if group.is_empty() || group.len() > 10 {
      return Err(BotError::invalid_command("/set_group", "/set_group [группа: длина &lt; 10]", "/set_group Ир3-21"));
    }

    let mut user = self.mongo.get_or_new(self.sender_id_i64()).await?;
    user.group = Some(group.clone());
    user.is_notifications_enabled = true;
    self.mongo.update(&user).await?;
    self
      .reply(format!("Теперь твоя группа: <code>{}</code>", user.group.unwrap()))
      .await?;
    Ok(())
  }

  pub async fn reply_default(&self, date: NaiveDate) -> BotResult {
    let group = match self.mongo.get_or_new(self.sender_id_i64()).await?.group {
      Some(g) => g,
      None => return self.reply("Ты не указал группу").await.map(|_| ()),
    };

    self
      .reply(api::default(group, date.weekday()).await.format(date))
      .await?;
    Ok(())
  }
}
