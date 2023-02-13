use async_trait::async_trait;
use maiq_shared::{utils::time::now, Fetch};
use teloxide::{macros::BotCommands, types::Message, Bot};

use crate::{
  bot::{context::Context, BotResult, Dispatch},
  db::MongoPool,
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

  #[command(description = "Стандартное расписание на сегодня")]
  DefaultToday,

  #[command(description = "Стандартное расписание на завтра")]
  DefaultNext,

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = "[группа] - Изменить группу")]
  SetGroup(String),

  #[command(description = "Старт")]
  Start,
}

#[async_trait]
impl Dispatch for Command {
  type Kind = Message;

  async fn dispatch(self, bot: Bot, kind: Self::Kind, mongo: MongoPool) -> BotResult {
    info!("Command {:?} from {} [{}]", self, kind.from().unwrap().full_name(), kind.from().unwrap().id.0);
    let ctx = Context::new(bot, kind, mongo);

    let result = match self {
      Command::Start => ctx.start().await,
      Command::About => ctx.reply_about().await,
      Command::ToggleNotifications => ctx.toggle_notifications().await,
      Command::SetGroup(ref group) => ctx.set_group(group).await,
      Command::Today => ctx.reply_timetable(Fetch::Today).await,
      Command::Next => ctx.reply_timetable(Fetch::Next).await,
      Command::DefaultToday => ctx.reply_default(now().date_naive()).await,
      Command::DefaultNext => ctx.reply_default(crate::bot::get_next_day()).await,
    };

    match result {
      Ok(_) => Ok(()),
      Err(err) => {
        error!("{}", err);
        ctx.reply(err.to_string()).await.map(|_| ())
      }
    }
  }
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,

  #[command(rename = "dev_userlist", description = "")]
  DevUserList,
}

#[async_trait]
impl Dispatch for DevCommand {
  type Kind = Message;

  async fn dispatch(self, bot: Bot, kind: Self::Kind, mongo: MongoPool) -> BotResult {
    type Cmd = DevCommand;
    let ctx = Context::new(bot, kind, mongo);

    match self {
      Cmd::DevNotifiables => ctx.reply(format!("{:?}", ctx.mongo.notifiables().await?)).await?,
      Cmd::DevUserList => ctx.dev_reply_user_list().await?,
    };
    Ok(())
  }
}
