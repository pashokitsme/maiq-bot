use api::ApiError;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use maiq_api_wrapper as api;
use maiq_shared::{
  default::{DefaultGroup, DefaultLesson},
  utils::time::now_date,
  Group, Lesson, Num, Snapshot,
};

use crate::error::{BotError, ReadableError};

pub trait SnapshotFormatter {
  fn format_group(&self, name: &str) -> Result<String, String>;
  fn format_teacher(&self, teacher: &str) -> String;
}

#[async_trait]
pub trait SnapshotFormatterExt {
  async fn format_or_default(&self, name: &str, date: NaiveDate) -> String;
}

pub trait DefaultFormatter {
  fn format(self, date: NaiveDate) -> String;
}

impl SnapshotFormatter for Snapshot {
  fn format_group(&self, name: &str) -> Result<String, String> {
    match self.group(name) {
      Some(group) => Ok(format_group(group, &self.uid, self.date)),
      None => Err(format!("Нет расписания для группы <b>{}</b> [<code>{}</code>]", name, self.uid)),
    }
  }

  fn format_teacher(&self, name: &str) -> String {
    let mut res = format!(
      "{} <b>{}</b> ({}) для <b>{}</b> [<code>{}</code>]\n\n",
      random_emoji(),
      self.date.date_naive().weekday_str_basic(),
      self.date.format("%d.%m.%Y"),
      name,
      self.uid
    );

    let push = |(group, lessons): (&String, &Vec<Lesson>)| {
      lessons
        .iter()
        .filter(|l| matches!(&l.teacher, Some(x) if x == name))
        .for_each(|l| res.push_str(&format!("<b>{}</b> · {}", group, format_lesson(l))))
    };

    self.groups.iter().map(|g| (&g.name, &g.lessons)).for_each(push);

    res
  }
}

#[async_trait]
impl SnapshotFormatterExt for Snapshot {
  async fn format_or_default(&self, name: &str, date: NaiveDate) -> String {
    let formatted = self.format_group(name);
    if let Ok(x) = formatted {
      return x;
    }

    let default = api::default(name, date.weekday()).await.format(date);
    match self.format_group(name) {
      Ok(x) => x,
      Err(x) => format!("{}\n\n{}", x, default),
    }
  }
}

impl DefaultFormatter for DefaultGroup {
  fn format(self, date: NaiveDate) -> String {
    let mut res = format!(
      "{} {}, {} - <b>стандартное</b> расписание <b>{}</b>\n\n",
      random_emoji(),
      date.weekday_str_basic(),
      date.format("%d.%m.%Y"),
      self.name
    );
    self.lessons.iter().for_each(|l| {
      if let Some(lesson) = &format_default_lesson(l, date.iso_week().week() % 2 != 0) {
        res.push_str(lesson)
      }
    });

    res
  }
}

impl DefaultFormatter for Result<DefaultGroup, ApiError> {
  fn format(self, date: NaiveDate) -> String {
    match self {
      Ok(d) => d.format(date),
      Err(err) => match &*err.cause {
        "default_not_found" => format!("Стандартное расписание не задано для {} 😒", date.weekday_str()),
        _ => {
          let err = BotError::from(err);
          error!("{err}");
          err.readable()
        }
      },
    }
  }
}

pub trait NaiveDateExt {
  fn weekday_str_basic(&self) -> &str;
  fn weekday_str(&self) -> &str;
}

impl NaiveDateExt for NaiveDate {
  fn weekday_str_basic(&self) -> &str {
    match self.weekday() {
      Weekday::Mon => "Понедельник",
      Weekday::Tue => "Вторник",
      Weekday::Wed => "Среда",
      Weekday::Thu => "Четверг",
      Weekday::Fri => "Пятница",
      Weekday::Sat => "Суббота",
      Weekday::Sun => "Воскресенье",
    }
  }

  fn weekday_str(&self) -> &str {
    match self.weekday() {
      Weekday::Mon => "понедельника",
      Weekday::Tue => "вторника",
      Weekday::Wed => "среды",
      Weekday::Thu => "четверга",
      Weekday::Fri => "пятницы",
      Weekday::Sat => "субботы",
      Weekday::Sun => "воскресенья",
    }
  }
}

fn format_group(group: &Group, snapshot_uid: &String, date: DateTime<Utc>) -> String {
  let mut res = match date == now_date() {
    true => format!(
      "{} {}, сегодня, {} [<code>{}</code>]\n\n",
      random_emoji(),
      date.date_naive().weekday_str_basic(),
      date.format("%d.%m.%Y"),
      snapshot_uid
    ),
    false => format!(
      "{} {}, {} [<code>{}</code>]\n\n",
      random_emoji(),
      date.date_naive().weekday_str_basic(),
      date.format("%d.%m.%Y"),
      snapshot_uid
    ),
  };

  group.lessons.iter().for_each(|l| res.push_str(&format_lesson(l)));
  res
}

fn format_lesson(lesson: &Lesson) -> String {
  let mut res = String::new();
  if let Num::Actual(ref num) = lesson.num {
    res.push_str(&format!("<b>#{}</b>", num))
  }

  if let Some(ref classroom) = lesson.classroom {
    res.push_str(&format!(" {}", classroom))
  }

  if let Some(ref sub) = lesson.subgroup {
    res.push_str(&format!(" · п/г <b>{}</b<", sub))
  };

  format!("{} <b>· {}</b>\n", res, lesson.name)
}

const EMOJIES: [&str; 21] =
  ["🥭", "🥩", "🥝", "🌵", "🥞", "🧀", "🍖", "🍌", "🍍", "🥓", "🧃", "🍒", "🍓", "🍇", "🥕", "🐷", "🍺", "🍪", "🍁", "🍉", "🍋"];

fn random_emoji<'a>() -> &'a str {
  EMOJIES[fastrand::usize(0..EMOJIES.len())]
}

fn format_default_lesson(lesson: &DefaultLesson, is_even_week: bool) -> Option<String> {
  let mut res = match lesson.is_even {
    Some(even) => match even == is_even_week {
      true => format!("<b>#{}</b>", lesson.num),
      false => return None,
    },
    None => format!("<b>#{}</b>", lesson.num),
  };

  res = match lesson.classroom.as_ref() {
    Some(classroom) => format!("{} {}", res, classroom),
    None => return Some(format!("{}<b> · {}</b>", res, lesson.name)),
  };
  res = match lesson.subgroup {
    Some(sub) => format!("{} · п/г {}", res, sub),
    None => res,
  };

  res = format!("{}<b> · {}</b>", res, lesson.name);

  res.push('\n');
  Some(res)
}
