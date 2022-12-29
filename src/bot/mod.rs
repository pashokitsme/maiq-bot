use teloxide::{
  dispatching::{HandlerExt, UpdateFilterExt},
  macros::BotCommands,
  prelude::Dispatcher,
  types::{Message, Update},
  Bot,
};

use crate::error::TeloxideError;

use self::context::HContext;

mod context;
pub mod start;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Command {
  Start,
}

pub async fn start(bot: Bot) {
  let msg_handler = Update::filter_message()
    .filter_command::<Command>()
    .endpoint(command_handler);

  Dispatcher::builder(bot, msg_handler)
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> Result<(), TeloxideError> {
  let ctx = HContext::new(&bot, &msg, cmd);
  ctx.say_hi().await?;
  Ok(())
}
