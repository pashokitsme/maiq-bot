use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt},
  dptree as dp,
  macros::BotCommands,
  prelude::Dispatcher,
  requests::Requester,
  types::{Message, Update},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{api, db::Mongo, error::BotError};

use self::handler::MContext;

mod handler;
mod timetable;

pub type BotResult = Result<(), BotError>;
pub type BotBodyResult = Result<String, BotError>;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum Command {
  #[command(description = "Старт")]
  Start,

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = "[группа: str] - Изменить группу")]
  SetGroup(String),

  #[command(description = "Расписание на сегодня")]
  Today,

  #[command(description = "Расписание на следующий день")]
  Next,

  #[command(description = "Информация")]
  About,
}

pub async fn start(bot: Bot, mongo: Mongo) {
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Couldn't set bot commands");

  let handler = Update::filter_message()
    .filter_command::<Command>()
    .branch(dp::endpoint(command_handler));

  info!("Started");
  Dispatcher::builder(bot, handler)
    .dependencies(dp::deps![mongo])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await
}

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> BotResult {
  let mut ctx = MContext::new(bot, msg, cmd, mongo);
  if let Err(err) = try_execute_command(&mut ctx).await {
    ctx.reply(err.to_string()).await?;
  }
  Ok(())
}

async fn try_execute_command(ctx: &mut MContext) -> BotResult {
  match ctx.used_command {
    Command::Start => ctx.start_n_init().await?,
    Command::About => ctx.reply_about().await?,
    Command::ToggleNotifications => ctx.toggle_notifications().await?,
    Command::SetGroup(ref group) => ctx.set_group(group).await?,
    Command::Today => send_single_timetable(ctx, false).await?,
    Command::Next => send_single_timetable(ctx, true).await?,
  }
  Ok(())
}

async fn send_single_timetable(ctx: &mut MContext, is_next: bool) -> BotResult {
  let group = match ctx.settings().await?.group.as_ref() {
    Some(x) => x.as_str(),
    None => return Err(BotError::NoTimetable),
  };

  let snapshot = match is_next {
    true => api::get_latest_next().await,
    false => api::get_latest_today().await,
  }?;

  let res = timetable::format_timetable(group, &snapshot).await?;
  ctx.reply(res).await?;
  Ok(())
}
