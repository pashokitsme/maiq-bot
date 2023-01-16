use chrono::{Datelike, NaiveDate, Weekday};
use maiq_shared::{utils, Fetch};
use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt},
  dptree as dp,
  macros::BotCommands,
  prelude::Dispatcher,
  requests::Requester,
  types::{Message, Update, UserId},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{
  api,
  db::{self, Mongo},
  env,
  error::BotError,
};

use self::{
  format::{SnapshotFormatter, SnapshotFormatterExt},
  handler::MContext,
};

pub mod notifier;

mod format;
mod handler;

pub type BotResult = Result<(), BotError>;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum Command {
  #[command(description = "Старт")]
  Start,

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = "[группа] - Изменить группу")]
  SetGroup(String),

  #[command(description = "Расписание на сегодня")]
  Today,

  #[command(description = "Расписание на следующий день")]
  Next,

  #[command(description = "Стандартное расписание на сегодня")]
  DefaultToday,

  #[command(description = "Стандартное расписание на завтра")]
  DefaultNext,

  #[command(description = "[uid] Получить снапшот")]
  Snapshot(String),

  #[command(description = "Информация")]
  About,
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,
}

pub async fn start(bot: Bot, mongo: Mongo) {
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Couldn't set bot commands");

  let me = bot.get_me().await.expect("Login error");
  let dev_id = UserId(env::parse_var(env::DEV_ID).unwrap_or(0));
  info!("Dev ID: {}", dev_id);
  let handler = Update::filter_message()
    .branch(dp::entry().filter_command::<Command>().endpoint(command_handler))
    .branch(
      dp::entry()
        .filter(move |msg: Message| msg.from().unwrap().id == dev_id)
        .filter_command::<DevCommand>()
        .endpoint(dev_command_handler),
    );

  bot.delete_webhook().await.expect("Couldn't delete webhook");
  info!("Logged in as {} [@{}]", me.full_name(), me.username());
  info!("Started");
  Dispatcher::builder(bot, handler)
    .dependencies(dp::deps![mongo])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

/*
? maybe will be used in future
async fn with_webhook(bot: Bot, url: Url, mut dispatcher: Dispatcher<Bot, BotError, DefaultKey>) {
  info!("Got webhook: {}", url);
  let listener = webhooks::axum(bot, webhooks::Options::new(([127, 0, 0, 1], 5500).into(), url))
    .await
    .expect("Couldn't start with webhook");

  dispatcher
    .dispatch_with_listener(listener, update_listener_error_handler)
    .await;
}
*/

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> BotResult {
  info!("Command {:?} from {} [{}]", cmd, msg.from().unwrap().full_name(), msg.from().unwrap().id.0);
  let mut ctx = MContext::new(bot, msg, cmd, mongo);
  if let Err(err) = try_execute_command(&mut ctx).await {
    error!("{}", err);
    ctx.reply(err.to_string()).await?;
  }
  Ok(())
}

pub async fn dev_command_handler(bot: Bot, msg: Message, cmd: DevCommand, mongo: Mongo) -> BotResult {
  match cmd {
    DevCommand::DevNotifiables => bot
      .send_message(msg.from().unwrap().id, format!("{:#?}", db::get_notifiables(&mongo).await?))
      .await
      .map(|_| ())?,
  }

  Ok(())
}

async fn try_execute_command(ctx: &mut MContext) -> BotResult {
  match ctx.used_command {
    Command::Start => ctx.start_n_init().await?,
    Command::About => ctx.reply_about().await?,
    Command::ToggleNotifications => ctx.toggle_notifications().await?,
    Command::SetGroup(ref group) => ctx.set_group(group).await?,
    Command::Today => send_single_timetable(ctx, Fetch::Today).await?,
    Command::Next => send_single_timetable(ctx, Fetch::Next).await?,
    Command::DefaultToday => ctx.reply_default(utils::now(0).date_naive()).await?,
    Command::DefaultNext => ctx.reply_default(get_next_day()).await?,
    Command::Snapshot(ref uid) => send_snapshot_to_user(ctx, uid).await?,
  }
  Ok(())
}

async fn send_single_timetable(ctx: &mut MContext, fetch: Fetch) -> BotResult {
  let group = ctx.settings().await?.group.unwrap_or("UNSET".into());
  let snapshot = api::latest(fetch.clone()).await;

  let date = match fetch {
    Fetch::Today => utils::now(0).date_naive(),
    Fetch::Next => get_next_day(),
  };

  let snapshot = match snapshot {
    Ok(s) => s,
    Err(e) => {
      ctx.reply(BotError::from(e).to_string()).await?;
      return ctx.reply_default(date).await;
    }
  };

  ctx.reply(snapshot.format_or_default(&*group, date).await).await
}

async fn send_snapshot_to_user(ctx: &MContext, uid: &String) -> BotResult {
  let settings = ctx.settings().await?;
  if uid.is_empty() || settings.group.is_none() {
    return Err(BotError::invalid_command("/snapshot", "/snapshot [uid]", "/snapshot aztc6qxcc3"));
  }

  //BotError::InvalidCommandUsage(
  //"Использование: <code>/snapshot [uid]</code>\nГруппа при этом должна быть указана\n\nПример: <code>/snapshot m010c556zk</code>".into(),
  //)

  let snapshot = api::snapshot(uid).await?;
  let body = match snapshot.format_group(&*settings.group.unwrap()) {
    Ok(x) => x,
    Err(x) => x,
  };
  ctx.reply(body).await
}

fn get_next_day() -> NaiveDate {
  let date = utils::now(1).date_naive();
  match date.weekday() == Weekday::Sun {
    true => utils::now(2).date_naive(),
    false => date,
  }
}
