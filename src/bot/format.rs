use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use maiq_shared::{
  default::{DefaultGroup, DefaultLesson},
  utils, Group, Lesson, Snapshot,
};

use crate::api::{self, ApiError, InnerPoll};

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
  Updated,
  New,
  Nothing,
}

pub trait SnapshotFormatter {
  fn format_group<'a>(&self, name: &'a str) -> Result<String, String>;
  fn lookup_changes<'g>(&'g self, prev: &Option<&InnerPoll>) -> HashMap<&'g str, Change>;
}

#[async_trait]
pub trait SnapshotFormatterExt {
  async fn format_or_default<'a>(&self, name: &'a str, date: NaiveDate) -> String;
}

pub trait DefaultFormatter {
  fn format(&self, date: NaiveDate) -> String;
}

impl SnapshotFormatter for Snapshot {
  fn format_group<'a>(&self, name: &'a str) -> Result<String, String> {
    match self.group(name) {
      Some(group) => Ok(format_group(group, &self.uid, self.date)),
      None => Err(format!("✖️ В снапшоте <code>{}</code> нет расписания для группы <b>{}</b>", self.uid, name)),
    }
  }

  fn lookup_changes<'g>(&'g self, prev: &Option<&InnerPoll>) -> HashMap<&'g str, Change> {
    if prev.is_none() {
      return self.groups.iter().map(|g| (g.name.as_str(), Change::New)).collect();
    }

    let prev = prev.unwrap();
    let mut result = HashMap::with_capacity(prev.groups.len());

    info!("Compating snapshot {} (prev was {})", self.uid, prev.uid);
    for group in self.groups.iter() {
      let prev = prev.groups.iter().find(|g| g.0.as_str() == group.name.as_str());
      let change = match (prev, group.uid.as_str()) {
        (Some(a), b) if a.1.as_str() != b => Change::Updated,
        (Some(a), b) if a.1.as_str() == b => Change::Nothing,
        (None, _) => Change::New,
        (Some(_), _) => unreachable!(),
      };
      info!("Comparing: {} >> {} & {:?} >> {:?}", group.name, group.uid, prev, change);

      result.insert(group.name.as_str(), change);
    }

    result
  }
}

#[async_trait]
impl SnapshotFormatterExt for Snapshot {
  async fn format_or_default<'a>(&self, name: &'a str, date: NaiveDate) -> String {
    let formatted = self.format_group(name);
    if let Ok(x) = formatted {
      return x;
    }

    let default = match api::default(name, date.weekday()).await {
      Ok(x) => x.format(date),
      Err(_) => format!("✖️ Нет стандартного расписания для группы <b>{}</b>, <code>{}</code>", name, date),
    };
    match self.format_group(name) {
      Ok(x) => x,
      Err(x) => format!("{}\n\n{}", x, default),
    }
  }
}

impl DefaultFormatter for DefaultGroup {
  fn format(&self, date: NaiveDate) -> String {
    let mut res = format!("Стандартное расписание <code>{}</code> на <code>{}</code>", self.name, date.weekday_str());
    let even = date.iso_week().week() % 2 != 0;
    match even {
      true => res.push_str("\nЧётная неделя\n\n"),
      false => res.push_str("\n<b>Не</b>чётная неделя\n\n"),
    };
    self.lessons.iter().for_each(|l| {
      if let Some(lesson) = &format_default_lesson(&l, even) {
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
      Err(_) => format!("✖️ Нет стандартного расписания"),
    }
  }
}

pub trait NaiveDateExt {
  fn weekday_str<'a>(&self) -> &'a str;
}

impl NaiveDateExt for NaiveDate {
  fn weekday_str<'a>(&self) -> &'a str {
    match self.weekday() {
      Weekday::Mon => "понедельник",
      Weekday::Tue => "вторник",
      Weekday::Wed => "среду",
      Weekday::Thu => "четверг",
      Weekday::Fri => "пятницу",
      Weekday::Sat => "субботу",
      Weekday::Sun => "воскресенье",
    }
  }
}

fn format_group(group: &Group, snapshot_uid: &String, date: DateTime<Utc>) -> String {
  let mut res = match date == utils::now_date(0) {
    true => format!(
      "Расписание <b>{}</b> на <code>сегодня</code>, <code>{}</code>\n[<code>{}</code>]\n\n",
      group.name,
      date.date_naive().weekday_str(),
      snapshot_uid
    ),
    false => {
      format!(
        "Расписание <b>{}</b> на <code>{}</code>, <code>{}</code>\n[<code>{}</code>]\n\n",
        group.name,
        date.format("%d.%m.%Y"),
        date.date_naive().weekday_str(),
        snapshot_uid
      )
    }
  };
  group
    .lessons
    .iter()
    .for_each(|l| res.push_str(&format_lesson_internal(&l)));
  res
}

fn format_lesson_internal(lesson: &Lesson) -> String {
  let mut res = format!("({}", lesson.num);
  res = match lesson.classroom.as_ref() {
    Some(classroom) => format!("{}, {})", res, classroom),
    None => return format!("{}) <b>{}</b>\n\n", res, lesson.name),
  };
  res = match lesson.subgroup {
    Some(sub) => format!("{} (п. {})", res, sub),
    None => res,
  };
  res = match lesson.teacher.as_ref() {
    Some(teacher) => format!("{} {}", res, teacher),
    None => res,
  };
  res = format!("{}\n<b>{}</b>\n\n", res, lesson.name);

  res
}

fn format_default_lesson(lesson: &DefaultLesson, is_even_week: bool) -> Option<String> {
  let mut res = match lesson.is_even {
    Some(even) => match even == is_even_week {
      true => format!("({}", lesson.num),
      false => return None,
    },
    None => format!("({}", lesson.num),
  };

  res = match lesson.classroom.as_ref() {
    Some(classroom) => format!("{}, {})", res, classroom),
    None => return Some(format!("{}) <b>{}</b>\n\n", res, lesson.name)),
  };
  res = match lesson.subgroup {
    Some(sub) => format!("{} (п. {})", res, sub),
    None => res,
  };
  res = match lesson.teacher.as_ref() {
    Some(teacher) => format!("{} {}", res, teacher),
    None => res,
  };
  res = format!("{}\n<b>{}</b>\n\n", res, lesson.name);

  Some(res)
}
