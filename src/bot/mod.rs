use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt},
  dptree as dp,
  prelude::Dispatcher,
  requests::Requester,
  types::Update,
  utils::command::BotCommands as _,
  Bot,
};

use crate::{db::Mongo, error::TeloxideError};
use commands::Command;

mod commands;

pub type TeloxideResult = Result<(), TeloxideError>;

pub async fn start(bot: Bot, mongo: Mongo) {
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Couldn't set bot commands");

  let handler = Update::filter_message()
    .filter_command::<Command>()
    .branch(dp::endpoint(commands::command_handler));

  info!("Started");
  Dispatcher::builder(bot, handler)
    .dependencies(dp::deps![mongo])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await
}
