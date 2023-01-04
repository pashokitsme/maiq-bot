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

// todo: await db & api requests in same time
// pub async fn update(&self) -> TeloxideResult {
//   let user = db::get_or_create_user(&self.mongo, self.user.id.0 as i64).await?;
//   if user.group.is_none() {
//     self.reply("Ты не установил группу").await?;
//     return Ok(());
//   }
//   let group = user.group.unwrap();

//   let today = api::get_latest_today().await;
//   let next = api::get_latest_next().await;
//   if today.is_err() && next.is_err() {
//     self.reply("У меня нет расписания ни на сегодня, ни на завтра. Может, стоит посмотреть на [сайте](http://chemk.org/index.php/raspisanie>)?").await?;
//     return Ok(());
//   }
//   let mut message = String::new();

//   if let Ok(today) = today {
//     if let Some(today) = today.groups.iter().find(|g| g.name == group) {
//       message.push_str("<b>Расписание на сегодня</b>\n");
//       today
//         .lessons
//         .iter()
//         .for_each(|l| message.push_str(utils::display_lesson(&l).as_str()));
//       message.push('\n');
//     }
//   }

//   if let Ok(next) = next {
//     if let Some(next) = next.groups.iter().find(|g| g.name == group) {
//       message.push_str("<b>Расписание на следующий день</b>\n");
//       next
//         .lessons
//         .iter()
//         .for_each(|l| message.push_str(utils::display_lesson(&l).as_str()))
//     }
//   }

//   self.reply(message).await?;
//   Ok(())
// }
