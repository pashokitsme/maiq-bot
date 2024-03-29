use async_trait::async_trait;
use maiq_shared::{utils::time::now, Fetch};
use teloxide::{macros::BotCommands, types::Message, Bot};

use crate::{
  bot::{context::Context, BotResult, Dispatch},
  db::MongoPool,
  error::ReadableError,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum Command {
  #[command(description = "Расписание на сегодня")]
  Today,

  #[command(description = "Расписание на следующий день")]
  Next,

  #[command(description = "Информация")]
  About,

  #[command(description = "Ссылки")]
  Links,

  #[command(description = "Стандартное расписание на сегодня")]
  DefaultToday,

  #[command(description = "Стандартное расписание на завтра")]
  DefaultNext,

  #[command(description = "Расписание на сегодня")]
  TeacherToday,

  #[command(description = "Расписание на завтра")]
  TeacherNext,

  #[command(description = "Получить расписание по дате")]
  Date(String),

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = " Изменить группу")]
  SelectGroup,

  #[command(description = "Установить имя")]
  SetTeacher(String),

  #[command(description = "Старт")]
  Start,
}

#[async_trait]
impl Dispatch for Command {
  type Kind = Message;

  async fn dispatch(&self, bot: Bot, kind: Self::Kind, mongo: MongoPool) -> BotResult {
    info!("Command {:?} from {} [{}]", self, kind.from().unwrap().full_name(), kind.from().unwrap().id.0);
    let ctx = Context::new(bot, kind, mongo);

    let res = match self {
      Command::Start => ctx.start().await,
      Command::About => ctx.reply_about().await,
      Command::Links => ctx.reply_links().await,
      Command::ToggleNotifications => ctx.toggle_notifications().await,
      Command::SelectGroup => ctx.reply_select_group().await,
      Command::Today => ctx.reply_timetable(Fetch::Today).await,
      Command::Next => ctx.reply_timetable(Fetch::Next).await,
      Command::DefaultToday => ctx.reply_default(now().date_naive()).await,
      Command::DefaultNext => ctx.reply_default(crate::bot::get_next_day()).await,
      Command::Date(date) => ctx.reply_dated_snapshot(date).await,
      Command::TeacherToday => ctx.reply_teacher_timetable(Fetch::Today).await,
      Command::TeacherNext => ctx.reply_teacher_timetable(Fetch::Next).await,
      Command::SetTeacher(ref name) => ctx.set_teacher(name).await,
    };

    if let Err(ref err) = res {
      ctx.reply(err.readable()).await?
    }

    res
  }
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,

  #[command(rename = "dev_userlist", description = "")]
  DevUserList,

  #[command(description = "")]
  Broadcast(String),
}

#[async_trait]
impl Dispatch for DevCommand {
  type Kind = Message;

  async fn dispatch(&self, bot: Bot, kind: Self::Kind, mongo: MongoPool) -> BotResult {
    let ctx = Context::new(bot, kind, mongo);

    match self {
      DevCommand::DevNotifiables => ctx.reply(format!("{:?}", ctx.mongo.notifiables().await?)).await?,
      DevCommand::DevUserList => ctx.dev_reply_user_list().await?,
      DevCommand::Broadcast(body) => ctx.dev_send_broadcast_agreement(body).await?,
    };
    Ok(())
  }
}
