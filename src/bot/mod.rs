use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Weekday};
use maiq_shared::{utils, Fetch};
use teloxide::{
  dispatching::{dialogue, HandlerExt, UpdateFilterExt, UpdateHandler},
  dptree as dp,
  payloads::{AnswerCallbackQuerySetters, SendMessageSetters},
  prelude::Dispatcher,
  requests::Requester,
  types::{CallbackQuery, InlineKeyboardMarkup, Message, Update, UserId},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{
  api,
  bot::{
    commands::{dev::DevCommand, user::Command},
    state::{GlobalState, GlobalStateStorage, State},
  },
  db::MongoPool,
  env,
  error::BotError,
};

use self::{
  callback::{Callback, CallbackKind},
  format::{SnapshotFormatter, SnapshotFormatterExt},
  handler::Context,
};

pub mod notifier;

mod callback;
mod commands;
mod format;
mod handler;
mod state;

lazy_static! {
  pub static ref DEV_ID: UserId = UserId(env::parse_var(env::DEV_ID).unwrap_or(0));
}

pub type BotResult = Result<(), BotError>;

#[async_trait]
trait Dispatch {
  async fn dispatch(self, bot: Bot, msg: Message, mongo: MongoPool, state: GlobalState) -> BotResult;
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
    .dependencies(dp::deps![GlobalStateStorage::new(), pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

fn dispatch_scheme() -> UpdateHandler<BotError> {
  use dp::case;

  info!("Dev ID: {}", *DEV_ID);
  let user_cmds_handler = Update::filter_message().branch(
    case![State::None]
      .branch(dp::entry().filter_command::<Command>().endpoint(dispatch::<Command>))
      .branch(
        dp::entry()
          .filter_command::<DevCommand>()
          .filter(move |msg: Message| msg.from().unwrap().id == *DEV_ID)
          .endpoint(dispatch::<DevCommand>),
      ),
  );

  let callback_handler = Update::filter_callback_query().endpoint(dispatch_callback_query);

  dialogue::enter::<Update, GlobalStateStorage, State, _>()
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

async fn dispatch<T: Dispatch>(dispatchable: T, bot: Bot, msg: Message, mongo: MongoPool, state: GlobalState) -> BotResult {
  dispatchable.dispatch(bot, msg, mongo, state).await
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
