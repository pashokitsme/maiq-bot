use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use maiq_shared::{
  default::{DefaultGroup, DefaultLesson},
  utils, Group, Lesson, Snapshot,
};

use crate::{
  api::{self, ApiError, InnerPoll},
  env,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
  Updated,
  New,
  Nothing,
  None,
}

pub trait SnapshotFormatter {
  fn format_group(&self, name: &str) -> Result<String, String>;
  fn lookup_changes(&self, prev: &Option<&InnerPoll>) -> HashMap<String, Change>;
}

#[async_trait]
pub trait SnapshotFormatterExt {
  async fn format_or_default(&self, name: &str, date: NaiveDate) -> String;
}

pub trait DefaultFormatter {
  fn format(&self, date: NaiveDate) -> String;
}

impl SnapshotFormatter for Snapshot {
  fn format_group(&self, name: &str) -> Result<String, String> {
    match self.group(name) {
      Some(group) => Ok(format_group(group, &self.uid, self.date)),
      None => Err(format!("В снапшоте <code>{}</code> нет расписания для группы <b>{}</b>", self.uid, name)),
    }
  }

  fn lookup_changes(&self, prev: &Option<&InnerPoll>) -> HashMap<String, Change> {
    let mut changes = env::var(env::GROUPS_LIST)
      .unwrap_or_default()
      .split(' ')
      .map(|g| (g.to_owned(), Change::None))
      .collect::<HashMap<String, Change>>();

    for group in self.groups.iter() {
      let prev = prev.and_then(|p| p.groups.iter().find(|g| g.0.as_str() == group.name.as_str()));
      let change = match (prev, group.uid.as_str()) {
        (Some(a), b) if a.1.as_str() != b => Change::Updated,
        (Some(a), b) if a.1.as_str() == b => Change::Nothing,
        (None, _) => Change::New,
        (Some(_), _) => unreachable!(),
      };

      changes.insert(group.name.clone(), change);
    }

    changes
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
  fn format(&self, date: NaiveDate) -> String {
    let mut res = format!(
      "[{}]   {}, {} - <b>стандартное</b> расписание <b>{}</b>\n\n",
      random_emoji(),
      date.weekday_str(),
      date.format("%d.%m.%Y"),
      self.name
    );
    self.lessons.iter().for_each(|l| {
      if let Some(lesson) = &format_default_lesson(&l, date.iso_week().week() % 2 != 0) {
        res.push_str(lesson)
      }
    });

    res
  }
}

impl DefaultFormatter for Result<DefaultGroup, ApiError> {
  fn format(&self, date: NaiveDate) -> String {
    match self {
      Ok(d) => d.format(date),
      Err(_) => format!("Нет стандартного расписания"),
    }
  }
}

pub trait NaiveDateExt {
  fn weekday_str(&self) -> &str;
}

impl NaiveDateExt for NaiveDate {
  fn weekday_str(&self) -> &str {
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
}

fn format_group(group: &Group, snapshot_uid: &String, date: DateTime<Utc>) -> String {
  let mut res = match date == utils::now_date(0) {
    true => format!("[{}]   Сегодня, {} [<code>{}</code>]\n\n", random_emoji(), date.format("%d.%m.%Y"), snapshot_uid),
    false => format!(
      "[{}]   {}, {} [<code>{}</code>]\n\n",
      random_emoji(),
      date.date_naive().weekday_str(),
      date.format("%d.%m.%Y"),
      snapshot_uid
    ),
  };
  group.lessons.iter().for_each(|l| res.push_str(&format_lesson(&l)));
  res
}

fn format_lesson(lesson: &Lesson) -> String {
  let mut res = match lesson.classroom.as_ref() {
    Some(classroom) => format!("<b>#{}</b> {}", lesson.num, classroom),
    None => return format!("<b>#{}</b> {}\n", lesson.num, lesson.name),
  };
  res = match lesson.subgroup {
    Some(sub) => format!("{} · п/г {}", res, sub),
    None => res,
  };
  res = format!("{}<b> · {}</b>\n", res, lesson.name);

  res
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
  res = match lesson.teacher.as_ref() {
    Some(t) => format!("{} · {}", res, t),
    None => res,
  };

  res.push('\n');
  Some(res)
}
