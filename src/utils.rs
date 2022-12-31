use maiq_structs::Lesson;

pub fn display_lesson<'a>(lesson: &Lesson) -> String {
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
