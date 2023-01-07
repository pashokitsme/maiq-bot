use maiq_structs::{Group, Lesson, Snapshot};

use crate::error::BotError;

use super::BotBodyResult;

pub async fn format_timetable<'g>(group_name: &'g str, snapshot: &Snapshot) -> BotBodyResult {
  snapshot
    .group(group_name)
    .ok_or(BotError::NoTimetable)
    .map(|g| display_group(&g))
}

fn display_group(group: &Group) -> String {
  let mut res = format!("Расписание: <b>{}</b>\n\n", group.name);
  for l in group.lessons.iter() {
    res.push_str(&display_lesson(&l))
  }

  res
}

fn display_lesson(lesson: &Lesson) -> String {
  let mut res = format!("\t[{}", lesson.num);
  res = if let Some(classroom) = lesson.classroom.as_ref() { format!("{}, {}", res, classroom) } else { format!("{}]", res) };
  res = if let Some(sub) = lesson.subgroup { format!("{} (п. {})", res, sub) } else { res };
  res = format!("{} {}", res, lesson.name);
  if let Some(teacher) = lesson.teacher.as_ref() {
    format!("{} [{}]", res, teacher)
  } else {
    res
  }
}
