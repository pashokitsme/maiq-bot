use async_trait::async_trait;
use maiq_shared::{utils, Fetch};
use teloxide::{macros::BotCommands, requests::Requester, types::Message, Bot};

use crate::{
  bot::{context::Context, BotResult, Dispatch, GlobalState},
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

  async fn dispatch(self, bot: Bot, kind: Self::Kind, mongo: MongoPool, _state: GlobalState) -> BotResult {
    info!("Command {:?} from {} [{}]", self, kind.from().unwrap().full_name(), kind.from().unwrap().id.0);
    let ctx = Context::new(bot, kind, self, mongo);

    let result = match ctx.used_command {
      Command::Start => ctx.start().await,
      Command::About => ctx.reply_about().await,
      Command::ToggleNotifications => ctx.toggle_notifications().await,
      Command::SetGroup(ref group) => ctx.set_group(group).await,
      Command::Today => ctx.reply_timetable(Fetch::Today).await,
      Command::Next => ctx.reply_timetable(Fetch::Next).await,
      Command::DefaultToday => ctx.reply_default(utils::now(0).date_naive()).await,
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
}

#[async_trait]
impl Dispatch for DevCommand {
  type Kind = Message;

  async fn dispatch(self, bot: Bot, kind: Self::Kind, mongo: MongoPool, _state: GlobalState) -> BotResult {
    match self {
      DevCommand::DevNotifiables => bot
        .send_message(kind.from().unwrap().id, format!("{:#?}", mongo.notifiables().await?))
        .await
        .map(|_| ())?,
    }
    Ok(())
  }
}
