use std::collections::HashMap;

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use maiq_shared::{
  default::{DefaultGroup, DefaultLesson},
  utils, Group, Lesson, Snapshot,
};

use crate::{api::InnerPoll, error::BotError};

use super::BotBodyResult;

/// Group = Message Body
pub fn separate_to_groups(snapshot: &Snapshot, prev: &Option<&InnerPoll>) -> HashMap<String, String> {
  snapshot
    .groups
    .iter()
    .filter_map(|g| {
      if let Some(prev) = prev {
        if let Some(prev_uid) = prev.groups.get(&g.name) {
          if prev_uid.as_str() == g.uid.as_str() {
            return None;
          }
        }
        return Some((g.name.clone(), display_group(&g, &snapshot.uid, snapshot.date)));
      }
      None
    })
    .collect::<HashMap<String, String>>()
}

pub fn format_timetable<'g>(group_name: &'g str, snapshot: &Snapshot) -> BotBodyResult {
  snapshot
    .group(group_name)
    .map(|g| display_group(&g, &snapshot.uid, snapshot.date))
    .ok_or(BotError::NoTimetableExpanded { group: group_name.into(), snapshot_uid: snapshot.uid.clone() })
}

pub fn display_default(default: DefaultGroup, date: NaiveDate) -> String {
  let mut res =
    format!("Стандартное расписание <code>{}</code> на <code>{}</code>", default.name, map_weekday_to_str(date.weekday()));
  let is_week_even = date.iso_week().week() % 2 != 0;
  match is_week_even {
    true => res.push_str("\nЧётная неделя\n\n"),
    false => res.push_str("\n<b>Не</b>чётная неделя\n\n"),
  };
  default.lessons.iter().for_each(|l| {
    if let Some(lesson) = &display_default_lesson(&l, is_week_even) {
      res.push_str(lesson)
    }
  });

  res
}

pub fn display_group(group: &Group, snapshot_uid: &String, date: DateTime<Utc>) -> String {
  let mut res = match date == utils::now_date(0) {
    true => format!(
      "Расписание <b>{}</b> на <code>сегодня</code>, <code>{}</code>\n[<code>{}</code>]\n\n",
      group.name,
      map_weekday_to_str(date.date_naive().weekday()),
      snapshot_uid
    ),
    false => {
      format!(
        "Расписание <b>{}</b> на <code>{}</code>, <code>{}</code>\n[<code>{}</code>]\n\n",
        group.name,
        date.format("%d.%m.%Y"),
        map_weekday_to_str(date.date_naive().weekday()),
        snapshot_uid
      )
    }
  };
  group.lessons.iter().for_each(|l| res.push_str(&display_lesson(&l)));
  res
}

fn display_lesson(lesson: &Lesson) -> String {
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

fn display_default_lesson(lesson: &DefaultLesson, is_even_week: bool) -> Option<String> {
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

fn map_weekday_to_str<'a>(weekday: Weekday) -> &'a str {
  match weekday {
    Weekday::Mon => "понедельник",
    Weekday::Tue => "вторник",
    Weekday::Wed => "среду",
    Weekday::Thu => "четверг",
    Weekday::Fri => "пятницу",
    Weekday::Sat => "субботу",
    Weekday::Sun => "воскресенье",
  }
}
