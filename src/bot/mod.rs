use teloxide::{
  dispatching::{dialogue::InMemStorage, HandlerExt, UpdateFilterExt},
  dptree as dp,
  macros::BotCommands,
  prelude::Dispatcher,
  requests::Requester,
  types::{Message, Update},
  utils::command::BotCommands as _,
  Bot,
};

use crate::{bot::dialogues::setup_settings, error::TeloxideError};

use self::handler_context::HContext;

mod dialogues;
mod handler_context;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Command {
  #[command(description = "start!")]
  Start,
}

pub async fn start(bot: Bot) {
  info!("Starting up!");
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Couldn't set bot commands");

  let msg_handler = Update::filter_message()
    .filter_command::<Command>()
    .branch(dp::case![Command::Start])
    .chain(setup_settings::build())
    .branch(dp::endpoint(command_handler));

  Dispatcher::builder(bot, msg_handler)
    .dependencies(dp::deps![InMemStorage::<setup_settings::State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> Result<(), TeloxideError> {
  let ctx = HContext::new(bot, msg, cmd);
  match ctx.used_command {
    Command::Start => ctx.say_hi().await?,
  }
  Ok(())
}
