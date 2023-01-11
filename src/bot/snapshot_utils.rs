use std::collections::HashMap;

use chrono::{DateTime, Datelike, Utc, Weekday};
use maiq_shared::{
  default::{DefaultGroup, DefaultLesson},
  utils, Group, Lesson, Snapshot,
};

use crate::api;

use super::BotBodyResult;

/// Group = Message Body
pub fn separate_to_groups(snapshot: &Snapshot) -> HashMap<String, String> {
  snapshot
    .groups
    .iter()
    .map(|g| (g.name.clone(), display_group(&g, &snapshot.uid, snapshot.date)))
    .collect::<HashMap<String, String>>()
}

pub async fn format_timetable<'g>(group_name: &'g str, snapshot: &Snapshot) -> BotBodyResult {
  let res = snapshot
    .group(group_name)
    .map(|g| display_group(&g, &snapshot.uid, snapshot.date));

  match res {
    Some(s) => Ok(s),
    None => {
      let day = snapshot.date.date_naive().weekday();
      let default = api::get_default(group_name, day).await?;
      Ok(display_default(default, day))
    }
  }
}

pub fn display_default(default: DefaultGroup, day: Weekday) -> String {
  let mut res = format!("Стандартное расписание <code>{}</code> <b>{}</b>\n\n", day, default.name);
  default
    .lessons
    .iter()
    .for_each(|l| res.push_str(&display_default_lesson(&l)));

  res
}

pub fn display_group(group: &Group, snapshot_uid: &String, date: DateTime<Utc>) -> String {
  let mut res = match date == utils::now_date(0) {
    true => format!("Расписание <b>{}</b> на <code>сегодня</code>\n[{}]\n\n", group.name, snapshot_uid),
    false => {
      format!("Расписание <b>{}</b> на <code>{}</code>\n[<code>{}</code>]\n\n", group.name, date.format("%d.%m.%Y"), snapshot_uid)
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

fn display_default_lesson(lesson: &DefaultLesson) -> String {
  let mut res = match lesson.is_even {
    Some(e) => match e {
      true => format!("(<code>Чёт.</code>, {}", lesson.num),
      false => format!("(<code>Нечёт.</code>, {}", lesson.num),
    },
    None => format!("({}", lesson.num),
  };

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
