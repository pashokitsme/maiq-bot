use std::collections::HashMap;

use maiq_structs::{Group, Lesson, Snapshot};

use crate::error::BotError;

use super::BotBodyResult;

pub fn get_formatted_snapshot(snapshot: &Snapshot) -> Result<HashMap<String, String>, BotError> {
  Ok(
    snapshot
      .groups
      .iter()
      .map(|g| (g.name.clone(), display_group(&g, &snapshot.uid)))
      .collect::<HashMap<String, String>>(),
  )
}

pub async fn format_timetable<'g>(group_name: &'g str, snapshot: &Snapshot) -> BotBodyResult {
  snapshot
    .group(group_name)
    .ok_or(BotError::NoTimetable)
    .map(|g| display_group(&g, &snapshot.uid))
}

fn display_group(group: &Group, snapshot_uid: &String) -> String {
  let mut res = format!("Расписание <b>{}</b>\t(<code>{}</code>)\n\n", group.name, snapshot_uid);
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
