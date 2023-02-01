use chrono::{Datelike, NaiveDate, Weekday};
use maiq_shared::{utils, Fetch};
use teloxide::{
  dispatching::{
    dialogue::{self, InMemStorage},
    HandlerExt, UpdateFilterExt, UpdateHandler,
  },
  dptree as dp,
  macros::BotCommands,
  payloads::{AnswerCallbackQuerySetters, SendMessageSetters},
  prelude::Dispatcher,
  requests::Requester,
  types::{CallbackQuery, InlineKeyboardMarkup, Message, Update, UserId},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{api, bot::state::State, db::MongoPool, env, error::BotError};

use self::{
  callback::{Callback, CallbackKind},
  format::{SnapshotFormatter, SnapshotFormatterExt},
  handler::Context,
};

pub mod notifier;

mod callback;
mod format;
mod handler;
mod state;

pub type BotResult = Result<(), BotError>;

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

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,
}

pub async fn start(bot: Bot, pool: MongoPool) {
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Couldn't set bot commands");

  let me = bot.get_me().await.expect("Login error");

  bot.delete_webhook().await.expect("Couldn't delete webhook");
  info!("Logged in as {} [@{}]", me.full_name(), me.username());
  info!("Started");

  Dispatcher::builder(bot, dispatch_scheme())
    .dependencies(dp::deps![InMemStorage::<State>::new(), pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

fn dispatch_scheme() -> UpdateHandler<BotError> {
  use dp::case;

  let dev_id = UserId(env::parse_var(env::DEV_ID).unwrap_or(0));
  info!("Dev ID: {}", dev_id);
  let user_cmds_handler = Update::filter_message().branch(
    case![State::None]
      .branch(dp::entry().filter_command::<Command>().endpoint(command_handler))
      .branch(
        dp::entry()
          .filter_command::<DevCommand>()
          .filter(move |msg: Message| msg.from().unwrap().id == dev_id)
          .endpoint(dev_command_handler),
      ),
  );

  let callback_handler = Update::filter_callback_query().endpoint(dispatch_callback_query);

  dialogue::enter::<Update, InMemStorage<State>, State, _>()
    .branch(user_cmds_handler)
    .branch(callback_handler)
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

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, mongo: MongoPool) -> BotResult {
  info!("Command {:?} from {} [{}]", cmd, msg.from().unwrap().full_name(), msg.from().unwrap().id.0);
  let mut ctx = Context::new(bot, msg, cmd, mongo);
  if let Err(err) = try_execute_command(&mut ctx).await {
    error!("{}", err);
    ctx.reply(err.to_string()).await?;
  }
  Ok(())
}

pub async fn dev_command_handler(bot: Bot, msg: Message, cmd: DevCommand, mongo: MongoPool) -> BotResult {
  match cmd {
    DevCommand::DevNotifiables => bot
      .send_message(msg.from().unwrap().id, format!("{:#?}", mongo.notifiables().await?))
      .await
      .map(|_| ())?,
  }

  Ok(())
}

pub async fn dispatch_callback_query(bot: Bot, q: CallbackQuery) -> BotResult {
  let kind: CallbackKind = match q
    .data
    .as_ref()
    .and_then(|data| bincode::deserialize(&data.as_bytes()).ok())
  {
    Some(x) => x,
    None => {
      bot
        .answer_callback_query(q.id)
        .text("Я не знаю, что делать с этой кнопкой 🤕")
        .show_alert(true)
        .await?;
      return Err(BotError::InvalidCallback);
    }
  };

  info!("Callback {:?} from {}", kind, q.from.full_name());

  callback::handle(bot, q, kind).await
}

async fn try_execute_command(ctx: &mut Context) -> BotResult {
  match ctx.used_command {
    Command::Hi => send_hi_btn(ctx).await?,
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

async fn send_hi_btn(ctx: &mut Context) -> BotResult {
  let markup =
    InlineKeyboardMarkup::new(vec![vec![Callback::new("123", CallbackKind::Ok), Callback::new("X", CallbackKind::Del)]]);

  ctx.send_message(ctx.chat_id(), "Хай").reply_markup(markup).await?;
  Ok(())
}

async fn send_single_timetable(ctx: &mut Context, fetch: Fetch) -> BotResult {
  let group = ctx
    .mongo
    .get_or_new(ctx.user_id())
    .await?
    .group
    .unwrap_or("UNSET".into());

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

async fn send_snapshot_to_user(ctx: &Context, uid: &String) -> BotResult {
  let settings = ctx.mongo.get_or_new(ctx.user_id()).await?;
  if uid.is_empty() || settings.group.is_none() {
    return Err(BotError::invalid_command("/snapshot", "/snapshot [uid]", "/snapshot aztc6qxcc3"));
  }

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
