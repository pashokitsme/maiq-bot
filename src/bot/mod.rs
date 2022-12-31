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

use crate::{db::Mongo, error::TeloxideError};

use self::handler::MContext;

mod handler;

pub type TeloxideResult = Result<(), TeloxideError>;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum Command {
  #[command(description = "Старт")]
  Start,

  #[command(description = "Включить/выключить уведомления")]
  ToggleNotifications,

  #[command(description = "Установить группу")]
  SetGroup(String),

  #[command(description = "123")]
  Update,

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

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> Result<(), TeloxideError> {
  let ctx = MContext::new(bot, msg, cmd, mongo);
  match ctx.used_command {
    Command::Start => ctx.start_command().await?,
    Command::About => ctx.send_about().await?,
    Command::ToggleNotifications => ctx.toggle_notifications().await?,
    Command::SetGroup(ref group) => ctx.set_group(group).await?,
    Command::Update => ctx.update().await?,
  }
  Ok(())
}
