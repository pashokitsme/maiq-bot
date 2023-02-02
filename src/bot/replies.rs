use maiq_api_models::{utils, Fetch};
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{api, bot::format::SnapshotFormatterExt, db::Settings};

use super::{context::Context, get_next_day, BotResult};

impl Context {
  pub async fn start(&self) -> BotResult {
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
          InlineKeyboardMarkup::new(vec![$(vec![InlineKeyboardButton::url($name, reqwest::Url::parse($url).unwrap()); 1]),*])
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
  
  · Если вместо названия пар написано <b>По расписанию</b>, значит нужно заполнить стандартное расписание, спрашивай <a href="https://t.me/pashokitsme">тут</a>.
  
  · Код проекта лежит на <a href="https://github.com/pashokitsme">гитхабе</a> и разделён на три репозитория:
      1. <a href="https://github.com/pashokitsme/maiq-parser">Парсер расписания</a>.
      2. <a href="https://github.com/pashokitsme/maiq-web-api">Бекенд</a>.
      Кстати, API публичное (но сервер, к сожалению, где-то в us west).
      3. <a href="https://github.com/pashokitsme/maiq-bot">Бот</a>.
  
  Жду пулл реквесты и/или звёздочки! 🌟
  "#,
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(markup)
        .await?;
    Ok(())
  }

  pub async fn reply_timetable(&self, fetch: Fetch) -> BotResult {
    let group = self
      .mongo
      .get_or_new(self.user_id())
      .await?
      .group
      .unwrap_or("UNSET".into());

    let date = match fetch {
      Fetch::Today => utils::now(0).date_naive(),
      Fetch::Next => get_next_day(),
    };

    match api::latest(fetch).await {
      Ok(s) => self.reply(s.format_or_default(&*group, date).await).await,
      Err(_) => self.reply_default(date).await,
    }
  }

  pub async fn dev_reply_user_list(&self) -> BotResult {
    let users = self.mongo.fetch_all().await?;
    let format = |u: &Settings| -> String {
      let r = match u.is_notifications_enabled {
        true => "[🟢] ",
        false => "[🔴] ",
      };

      format!(
        "{} {} [<a href=\"tg://user?id={}\">#{}</a>] с {}\n",
        r,
        u.group.as_ref().unwrap_or(&"no".into()),
        u.id,
        u.id,
        u.joined.to_chrono().format("%d/%m/%Y %H:%M:%S")
      )
    };

    let body = format!("Всего: <b>{}</b>\n\n {}", users.len(), users.iter().map(format).collect::<String>());

    self.reply(body).await?;
    Ok(())
  }
}
