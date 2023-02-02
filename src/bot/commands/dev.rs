use async_trait::async_trait;
use teloxide::{macros::BotCommands, requests::Requester, types::Message, Bot};

use crate::{
  bot::{state::GlobalState, BotResult, Dispatch},
  db::MongoPool,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,
}

#[async_trait]
impl Dispatch for DevCommand {
  async fn dispatch(self, bot: Bot, msg: Message, mongo: MongoPool, _state: GlobalState) -> BotResult {
    match self {
      DevCommand::DevNotifiables => bot
        .send_message(msg.from().unwrap().id, format!("{:#?}", mongo.notifiables().await?))
        .await
        .map(|_| ())?,
    }
    Ok(())
  }
}
