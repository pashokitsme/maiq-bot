use teloxide::{macros::BotCommands, types::Message, Bot};

mod handler;

use crate::{db::Mongo, error::TeloxideError};
use handler::MContext;

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

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, mongo: Mongo) -> Result<(), TeloxideError> {
  let ctx = MContext::new(bot, msg, cmd, mongo);
  match ctx.used_command {
    Command::Start => (),
    Command::About => ctx.send_about().await?,
    Command::ToggleNotifications => ctx.toggle_notifications().await?,
    Command::SetGroup(ref group) => ctx.set_group(group).await?,
    Command::Update => ctx.update().await?,
  }
  Ok(())
}
