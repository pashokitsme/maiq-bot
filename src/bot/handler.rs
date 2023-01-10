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
  bot::BotResult,
  db::{self, Mongo},
  error::BotError,
};

// todo: (?) make it injectable
/// M`essage` handler context
pub struct MContext {
  bot: Bot,
  pub msg: Message,
  pub user: User,
  pub used_command: Command,
  pub mongo: Mongo,
  user_settings: Option<db::UserSettings>,
}

impl Deref for MContext {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl MContext {
  pub fn new(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> Self {
    Self { bot, user: msg.from().unwrap().clone(), msg, used_command: cmd, mongo, user_settings: None }
  }

  pub fn chat_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub async fn reply<T: Into<String>>(&self, text: T) -> Result<Message, BotError> {
    Ok(
      self
        .bot
        .send_message(self.chat_id(), text)
        .parse_mode(ParseMode::Html)
        .disable_web_page_preview(true)
        .await?,
    )
  }

  pub async fn settings<'u>(&'u mut self) -> Result<&'u db::UserSettings, BotError> {
    if let None = self.user_settings {
      self.user_settings = Some(db::get_or_create_user_settings(&self.mongo, self.user.id.0 as i64).await?);
    }
    Ok(&self.user_settings.as_ref().unwrap())
  }

  pub async fn start_n_init(&self) -> BotResult {
    _ = db::get_or_create_user_settings(&self.mongo, self.user.id.0 as i64).await?;
    self
      .reply(
        r#"Привет. Это что-то типо беты. По всем вопросам/багам/предложениям <a href=\"https://t.me/pashokitsme\">сюда</a>.
        
        Кстати, в поиске хоста.
        
        Для начала тебе нужно установить свою группу:\n<code>/set_group [группа: str]</code>"#,
      )
      .await?;
    Ok(())
  }

  pub async fn reply_about(&self) -> BotResult {
    macro_rules! buttons_column {
      ($(($name: literal, $url: literal)),*) => {
        vec![$(vec![InlineKeyboardButton::url($name, Url::parse($url).unwrap()); 1]),*]
      };
    }

    let markup = InlineKeyboardMarkup::new(buttons_column!(
      ("По всем вопросам", "https://t.me/pashokitsme"),
      ("API", "https://github.com/pashokitsme/maiq-web-api"),
      ("GitHub", "https://github.com/pashokitsme")
    ));
    let msg = format!(
      r#"<b>Информация</b>
      
      Заглушка :("#
    );
    self
      .send_message(self.chat_id(), msg)
      .parse_mode(ParseMode::Html)
      .reply_markup(markup)
      .await?;
    Ok(())
  }

  pub async fn toggle_notifications(&self) -> BotResult {
    let mut user = db::get_or_create_user_settings(&self.mongo, self.user.id.0 as i64).await?;
    user.is_notifications_enabled = !user.is_notifications_enabled;
    db::update_user_settings(&self.mongo, &user).await?;
    self.reply(format!("{}", user.is_notifications_enabled)).await?;
    Ok(())
  }

  pub async fn set_group(&self, group: &String) -> BotResult {
    if group.is_empty() || group.len() > 10 {
      return Err(BotError::InvalidCommandUsage(
        "Использование команды:\n<code>/set_group [группа: str, длина &lt; 10]</code>\nПример:\n<code>/set_group Ир3-21</code>"
          .into(),
      ));
    }
    let mut user = db::get_or_create_user_settings(&self.mongo, self.user.id.0 as i64).await?;
    user.group = Some(group.clone());
    user.is_notifications_enabled = true;
    db::update_user_settings(&self.mongo, &user).await?;
    self
      .reply(format!("Теперь твоя группа: <code>{}</code>", user.group.unwrap()))
      .await?;
    Ok(())
  }
}
