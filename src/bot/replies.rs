use chrono::{Datelike, NaiveDate};
use maiq_api_wrapper::api;
use maiq_shared::{utils::time::now, Fetch};
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

use crate::{
  bot::format::{SnapshotFormatter, SnapshotFormatterExt},
  db::Settings,
  error::BotError,
};

use super::{
  callbacks::{Callback, CallbackKind},
  context::Context,
  format::DefaultFormatter,
  get_next_day, BotResult,
};

impl Context {
  pub async fn start(&self) -> BotResult {
    self.mongo.get_or_new(self.user_id()).await?;
    let username = &self.msg.from().unwrap().first_name;
    self
      .reply(format!(
        r#"Привет, <b>{username}</b>! 🎉
На данный момент это что-то типо беты.
Желаешь помочь? В /about есть ссылка на гитхаб.

По всем вопросам/багам/предложениям пиши <a href="https://t.me/pashokitsme">ему</a>.

Как пользоваться ботом, информация о нём, ссылки - всё так же: /about"#
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
  
  · Код проекта лежит на <a href="https://github.com/pashokitsme">гитхабе</a> и разделён на 3 репозитория. Жду 🌟 и/или пулл реквесты 

  · Изменить свою группу можно при помощи команды /set_group. В аргументе нужно указать её название, такое же, как и в расписании. 

  · Можно отключить/включить уведомления при помощи /toggle_notifications.

  · Бота можно добавить в чат, команды работать будут, но уведомления - нет
  
  · Если вместо названия пар написано <b>По расписанию</b>, значит нужно заполнить стандартное расписание, спрашивай <a href="https://t.me/pashokitsme">тут</a>.

  · [<code>g5q98alka3</code>] - это уникальный ID расписания (всего или группы, показывается всего), по нему определяются изменения. В конечном итоге за один день остаётся одна запись последней версии, другие (после 29.01.23) заменяются. По сути - уже не очень то нужнен для отображения.
  "#,
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(markup)
        .await?;
    Ok(())
  }

  pub async fn reply_timetable(&self, fetch: Fetch) -> BotResult {
    let group = self.mongo.get_or_new(self.user_id()).await?.group;

    let group = match group {
      Some(g) => g,
      None => return self.reply("Ты не указал группу").await.map(|_| ()),
    };

    let date = match fetch {
      Fetch::Today => now().date_naive(),
      Fetch::Next => get_next_day(),
    };

    match api::latest(fetch).await {
      Ok(s) => self.reply(s.format_or_default(&group, date).await).await,
      Err(_) => self.reply_default(date).await,
    }
  }

  pub async fn reply_default(&self, date: NaiveDate) -> BotResult {
    match self.mongo.get_or_new(self.user_id()).await?.group {
      Some(g) => self.reply(api::default(&g, date.weekday()).await.format(date)).await,
      None => self.reply("Ты не указал группу").await.map(|_| ()),
    }
  }

  pub async fn reply_dated_snapshot(&self, rawdate: &str) -> BotResult {
    fn parse_date(rawdate: &str) -> Result<NaiveDate, ()> {
      let mut slice = rawdate.split('.');
      macro_rules! parse {
        () => {
          slice.next().and_then(|x: &str| x.parse().ok()).ok_or(())?
        };
      }
      let (d, m, y) = (parse!(), parse!(), parse!());
      NaiveDate::from_ymd_opt(y, m, d).ok_or(())
    }

    let group = match self.mongo.get_or_new(self.user_id()).await?.group {
      Some(g) => g,
      None => return self.reply("Ты не указал группу").await.map(|_| ()),
    };

    let date =
      parse_date(rawdate).map_err(|_| BotError::invalid_command("/date", "/date [дата в формате d.m.Y]", "/date 11.02.2023"))?;

    let r = match api::date(date).await?.format_group(&group) {
      Ok(r) => r,
      Err(r) => r,
    };
    self.reply(r).await?;

    Ok(())
  }

  pub async fn dev_reply_user_list(&self) -> BotResult {
    let users = self.mongo.fetch_all().await?;
    let format = |u: &Settings| -> String {
      let r = match u.is_notifications_enabled {
        true => "[+] ",
        false => "[-] ",
      };

      format!(
        "{} {} [<a href=\"tg://user?id={}\">#{}</a>] с {}\n",
        r,
        u.group.as_ref().unwrap_or(&"-".into()),
        u.id,
        u.id,
        u.joined.to_chrono().format("%d/%m/%Y %H:%M:%S")
      )
    };

    let body = format!("Всего: <b>{}</b>\n\n{}", users.len(), users.iter().map(format).collect::<String>());

    self.reply(body).await?;
    Ok(())
  }

  pub async fn dev_send_broadcast_agreement(&self, body: &String) -> BotResult {
    if body.is_empty() {
      return self.reply("Сообщение пустое").await.map(|_| ());
    }
    let buttons = vec![vec![Callback::button("X", CallbackKind::Del), Callback::button("OK", CallbackKind::SendBroadcast)]];
    let reply_markup = InlineKeyboardMarkup::new(buttons);
    self
      .send_message(self.user_id(), body)
      .reply_markup(reply_markup)
      .await?;

    self
      .send_message(self.user_id(), format!("Превью:\n{}", body))
      .parse_mode(ParseMode::Html)
      .reply_markup(InlineKeyboardMarkup::new(vec![vec![Callback::button("X", CallbackKind::Del)]]))
      .await?;
    Ok(())
  }
}
