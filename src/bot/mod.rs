use async_trait::async_trait;

use maiq_shared::utils::time::now_with_offset;
use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt, UpdateHandler},
  dptree as dp,
  prelude::Dispatcher,
  requests::Requester,
  types::{CallbackQuery, Message, Update, UserId},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{
  bot::{
    callbacks::CallbackKind,
    commands::{Command, DevCommand},
  },
  db::MongoPool,
  env,
  error::BotError,
};

pub mod notifier;

mod callbacks;
mod commands;
mod context;
mod format;
mod replies;

lazy_static! {
  pub static ref DEV_ID: UserId = UserId(env::parse_var(env::DEV_ID).unwrap_or(0));
}

pub type BotResult = Result<(), BotError>;

#[async_trait]
trait Dispatch {
  type Kind;

  async fn dispatch(&self, bot: Bot, kind: Self::Kind, mongo: MongoPool) -> BotResult;
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
    .dependencies(dp::deps![pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

fn dispatch_scheme() -> UpdateHandler<BotError> {
  info!("Dev ID: {}", *DEV_ID);
  let cmds_handler = Update::filter_message()
    .branch(
      dp::entry()
        .filter_command::<Command>()
        .endpoint(dispatch::<Command, Message>),
    )
    .branch(
      dp::entry()
        .filter_command::<DevCommand>()
        .filter(move |msg: Message| msg.from().unwrap().id == *DEV_ID)
        .endpoint(dispatch::<DevCommand, Message>),
    )
    .endpoint(unhandled_message);

  let callback_handler = Update::filter_callback_query().endpoint(dispatch_query);

  dp::entry()
    .branch(cmds_handler)
    .branch(callback_handler)
    .endpoint(unhandled_update)
}

async fn dispatch_query(bot: Bot, query: CallbackQuery, mongo: MongoPool) -> BotResult {
  let kind: CallbackKind = query
    .data
    .as_ref()
    .and_then(|data| bincode::deserialize(data.as_bytes()).ok())
    .unwrap_or(CallbackKind::Unknown);

  info!("Callback {:?} from {}", kind, query.from.full_name());
  dispatch(kind, bot, query, mongo).await
}

async fn unhandled_message(msg: Message) -> BotResult {
  if let Some(user) = msg.from() {
    warn!(
      "Unhandled message from: {name} [@{username} #{id}] {chat_kind:?}]\nKind: {kind:?}",
      name = user.full_name(),
      username = user.username.as_deref().unwrap_or("?"),
      id = user.id.0,
      chat_kind = msg.chat.kind,
      kind = msg.kind
    );
  }
  Ok(())
}

async fn unhandled_update(update: Update) -> BotResult {
  if let Some(user) = update.user() {
    warn!(
      "Unhandled update #{update_id} from {name} [@{username} #{id}]",
      update_id = update.id,
      name = user.full_name(),
      username = user.username.as_deref().unwrap_or("?"),
      id = user.id.0
    );
  }

  Ok(())
}

async fn dispatch<T: Dispatch<Kind = K>, K>(dispatchable: T, bot: Bot, kind: K, db: MongoPool) -> BotResult {
  if let Err(ref err) = dispatchable.dispatch(bot, kind, db).await {
    error!("{err}");
  }
  Ok(())
}

fn get_next_day() -> chrono::NaiveDate {
  let date = now_with_offset(1).date_naive();
  match chrono::Datelike::weekday(&date) == chrono::Weekday::Sun {
    true => now_with_offset(2).date_naive(),
    false => date,
  }
}
