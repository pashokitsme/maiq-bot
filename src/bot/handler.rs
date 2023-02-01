use chrono::{Datelike, NaiveDate};
use reqwest::Url;
use std::ops::Deref;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode, User, UserId},
  Bot,
};

use super::{format::DefaultFormatter, Command};
use crate::{api, bot::BotResult, db::MongoPool, error::BotError};

pub struct Context {
  bot: Bot,
  pub msg: Message,
  pub user: User,
  pub used_command: Command,
  pub mongo: MongoPool,
}

impl Deref for Context {
  type Target = Bot;

  fn deref(&self) -> &Self::Target {
    &self.bot
  }
}

impl Context {
  pub fn new(bot: Bot, msg: Message, cmd: Command, mongo: MongoPool) -> Self {
    Self { bot, user: msg.from().unwrap().clone(), msg, used_command: cmd, mongo }
  }

  pub fn chat_id(&self) -> ChatId {
    self.msg.chat.id
  }

  pub fn user_id(&self) -> UserId {
    self.msg.from().and_then(|f| Some(f.id)).unwrap_or(UserId(0))
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

  pub async fn start_n_init(&self) -> BotResult {
    self.mongo.get_or_new(self.user_id()).await?;
    let username = &self.user.first_name;
    self
      .reply(format!(
        r#"Привет, <b>{username}</b>! 🎉
На данный момент это что-то типо беты.
Желаешь помочь? В /about есть ссылка на гитхаб.

По всем вопросам/багам/предложениям пиши <a href="https://t.me/pashokitsme">ему</a>.

Как пользоваться ботом, информация о нём, ссылки:
<b>/about</b>"#
      ))
      .await?;
    Ok(())
  }

  pub async fn reply_about(&self) -> BotResult {
    macro_rules! url_buttons_column {
      ($(($name: literal, $url: literal)),*) => {
        InlineKeyboardMarkup::new(vec![$(vec![InlineKeyboardButton::url($name, Url::parse($url).unwrap()); 1]),*])
      };
    }

    let markup = url_buttons_column!(("API docs", "https://github.com/pashokitsme/maiq-web-api"));

    self
      .send_message(
        self.chat_id(),
        r#"<b>Информация</b>

· Изменить свою группу можно при помощи команды /set_group. 
В аргументе нужно указать её название, такое же, как и в расписании. 

· Можно отключить/включить уведомления при помощи
    /toggle_notifications.

· [<code>g5q98alka3</code>] - это уникальный ID снапшота или группы, по нему определяются изменения в расписании. В конечном итоге за один день остаётся один снапшот последней версии, другие теперь (после 29.01.23) заменяются.

· Код проекта лежит на <a href="https://github.com/pashokitsme">гитхабе</a> и разделён на три репозитория:
    1. <a href="https://github.com/pashokitsme/maiq-parser">Парсер расписания</a>.
    2. <a href="https://github.com/pashokitsme/maiq-web-api">Бекенд</a>.
    Кстати, API публичное (но сервер, к сожалению, где-то в us west) - если будет желание использовать, <a href="https://t.me/pashokitsme">пишите</a>, интересно же.
    3. <a href="https://github.com/pashokitsme/maiq-bot">Бот</a>.

Жду пулл реквесты и/или звёздочки! 🌟
"#,
      )
      .parse_mode(ParseMode::Html)
      .reply_markup(markup)
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

  pub async fn reply_default(&self, date: NaiveDate) -> BotResult {
    match self.mongo.get_or_new(self.user_id()).await?.group {
      Some(g) => self.reply(api::default(g, date.weekday()).await.format(date)).await,
      None => self.reply("Ты не указал группу").await.map(|_| ()),
    }
  }
}
