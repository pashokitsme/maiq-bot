use async_trait::async_trait;
use maiq_shared::{utils, Fetch};
use teloxide::{macros::BotCommands, types::Message, Bot};

use crate::{
  bot::{handler::Context, state::GlobalState, BotResult, Dispatch},
  db::MongoPool,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum Command {
  #[command(description = "Hi!")]
  Hi,

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

  #[command(description = "[uid] Получить снапшот")]
  Snapshot(String),

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = "[группа] - Изменить группу")]
  SetGroup(String),

  #[command(description = "Старт")]
  Start,
}

#[async_trait]
impl Dispatch for Command {
  async fn dispatch(self, bot: Bot, msg: Message, mongo: MongoPool, _state: GlobalState) -> BotResult {
    info!("Command {:?} from {} [{}]", self, msg.from().unwrap().full_name(), msg.from().unwrap().id.0);
    let mut ctx = Context::new(bot, msg, self, mongo);
    if let Err(err) = try_execute_command(&mut ctx).await {
      error!("{}", err);
      ctx.reply(err.to_string()).await?;
    }
    Ok(())
  }
}

async fn try_execute_command(ctx: &mut Context) -> BotResult {
  match ctx.used_command {
    Command::Hi => crate::bot::send_hi_btn(ctx).await?,
    Command::Start => ctx.start_n_init().await?,
    Command::About => ctx.reply_about().await?,
    Command::ToggleNotifications => ctx.toggle_notifications().await?,
    Command::SetGroup(ref group) => ctx.set_group(group).await?,
    Command::Today => crate::bot::send_single_timetable(ctx, Fetch::Today).await?,
    Command::Next => crate::bot::send_single_timetable(ctx, Fetch::Next).await?,
    Command::DefaultToday => ctx.reply_default(utils::now(0).date_naive()).await?,
    Command::DefaultNext => ctx.reply_default(crate::bot::get_next_day()).await?,
    Command::Snapshot(ref uid) => crate::bot::send_snapshot_to_user(ctx, uid).await?,
  }

  Ok(())
}
